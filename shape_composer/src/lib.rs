//! Vision-guided primitive scene composer.
//!
//! The crate intentionally keeps the control loop small: the decision model writes
//! one scene, the perception model judges its PNG, and the runner archives every
//! version before repeating.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use image::{ImageBuffer, Rgb};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::Command, time::{timeout, Duration}};
use uuid::Uuid;

pub const MAX_ITERATIONS: u32 = 60;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode { Auto, Hil }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunRequest {
    pub goal: String,
    #[serde(default = "default_mode")]
    pub mode: Mode,
    #[serde(default = "default_iterations")]
    pub max_iterations: u32,
    pub seed_prompt: Option<String>,
    #[serde(default = "default_reference_image")]
    pub reference_image: PathBuf,
}
fn default_mode() -> Mode { Mode::Auto }
fn default_iterations() -> u32 { 25 }
fn default_reference_image() -> PathBuf { PathBuf::from("image.png") }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict { GoalAchieved, Partial, Wrong }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Measurement { pub label: String, pub action: String, pub percent: Option<f32> }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PerceptionOutput {
    pub verdict: Verdict,
    #[serde(default)]
    pub critiques: Vec<String>,
    #[serde(default)]
    pub measurements: Vec<Measurement>,
    pub suggestion_rank: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCall { pub name: String, pub arguments: Value }
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecisionOutput {
    pub reasoning: String,
    pub tool_calls: Vec<ToolCall>,
    pub self_critique: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryItem {
    pub id: String, pub kind: String, pub keywords: Vec<String>, pub descriptor: String,
    pub value: Value, pub source: String, pub run_id: String, pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IterationEvent {
    pub run_id: String, pub iteration: u32, pub phase: String, pub payload: Value, pub ts: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunSummary {
    pub run_id: String, pub goal: String, pub mode: Mode, pub iterations: u32,
    pub final_verdict: Verdict, pub final_png: Option<String>, pub duration_s: f64,
    pub memory_items_written: usize,
}

#[derive(Clone)]
pub struct Gateway {
    client: reqwest::Client,
    base_url: String,
}
impl Gateway {
    pub fn from_env() -> Result<Self> {
        Ok(Self { client: reqwest::Client::builder().timeout(Duration::from_secs(120)).build()?,
            base_url: std::env::var("LLM_GATEWAY_URL").unwrap_or_else(|_| "http://localhost:8108".into()) })
    }
    async fn chat(&self, body: Value) -> Result<Value> {
        let response = self.client.post(format!("{}/v1/chat", self.base_url))
            .json(&body).send().await.context("LLM gateway unavailable")?;
        if !response.status().is_success() { bail!("gateway returned {}: {}", response.status(), response.text().await.unwrap_or_default()); }
        Ok(response.json().await?)
    }
    /// Plain-text completion: Gemini's structured-output mode silently drops or
    /// misplaces long string fields (the full Rust program), so the decision agent
    /// answers in prose + one ```rust fence and we parse the fence ourselves.
    pub async fn decision(&self, prompt: String, session: &str) -> Result<DecisionOutput> {
        let response = self.chat(json!({"prompt": prompt, "provider":"gemini", "agent":"composer-decision",
            "session":session, "temperature":0.2, "max_tokens":8000})).await?;
        let text = response.get("text").and_then(Value::as_str)
            .ok_or_else(|| anyhow!("gateway did not return text for the decision"))?;
        parse_decision_text(text)
    }
    pub async fn perceive(&self, prompt: String, render_png: &[u8], reference_png: &[u8], schema: Value, session: &str) -> Result<PerceptionOutput> {
        let render_url = format!("data:image/png;base64,{}", BASE64.encode(render_png));
        let reference_url = format!("data:image/png;base64,{}", BASE64.encode(reference_png));
        let body = json!({"messages":[{"role":"user","content":[
            {"type":"text","text":prompt},
            {"type":"image_url","image_url":{"url":render_url}},
            {"type":"image_url","image_url":{"url":reference_url}}
        ]}],
          "provider":"gemini","agent":"composer-perception","session":session,"temperature":0.1,"max_tokens":1200,
          "response_format":{"type":"json_schema","name":"PerceptionOutput","strict":true,"schema":schema}});
        let response = self.chat(body).await?;
        eprintln!(
            "[perception] gateway model={} input_tokens={}",
            response.get("model").and_then(Value::as_str).unwrap_or("unknown"),
            response.get("input_tokens").and_then(Value::as_u64)
                .map(|value| value.to_string()).unwrap_or_else(|| "unknown".into()),
        );
        let payload = response.get("parsed").cloned().or_else(|| {
            response.get("text").and_then(Value::as_str).and_then(|text| serde_json::from_str(text).ok())
        }).ok_or_else(|| anyhow!("gateway did not return a PerceptionOutput payload"))?;
        serde_json::from_value(payload).context("gateway did not return a valid PerceptionOutput")
    }
}

pub fn workspace_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf() }
pub fn state_root() -> PathBuf { std::env::var_os("SHAPE_COMPOSER_STATE_DIR").map(PathBuf::from).unwrap_or_else(|| workspace_root().join("state")) }
pub fn version_name(n: u32) -> String { format!("v{n:02}") }
fn now() -> String { Utc::now().to_rfc3339() }
async fn emit_event(run_id: &str, iteration: u32, phase: &str, payload: Value) {
    let base = std::env::var("LIVE_SERVER_URL").unwrap_or_else(|_| "http://localhost:3030".into());
    let event = IterationEvent { run_id: run_id.into(), iteration, phase: phase.into(), payload, ts: now() };
    let _ = reqwest::Client::new().post(format!("{base}/agent/ingest/{run_id}")).json(&event).send().await;
}
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let temp = path.with_extension(format!("{}.tmp", path.extension().and_then(|s| s.to_str()).unwrap_or("tmp")));
    fs::write(&temp, content)?; fs::rename(temp, path)?; Ok(())
}

pub fn safe_resolve(root: &Path, requested: &str) -> Result<PathBuf> {
    let relative = Path::new(requested);
    if relative.is_absolute() || relative.components().any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
        bail!("unsafe path: {requested}");
    }
    let joined = root.join(relative);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if joined.exists() && !joined.canonicalize()?.starts_with(&canonical_root) { bail!("path escapes run directory"); }
    Ok(joined)
}

pub fn ppm_to_png(input: &Path, output: &Path) -> Result<(u32, u32)> {
    let text = fs::read_to_string(input)?;
    let tokens: Vec<_> = text.lines().flat_map(|l| l.split('#').next().unwrap_or("").split_whitespace()).collect();
    if tokens.len() < 4 || tokens[0] != "P3" { bail!("only ASCII P3 PPM is supported"); }
    let width: u32 = tokens[1].parse()?; let height: u32 = tokens[2].parse()?; let max: u32 = tokens[3].parse()?;
    if max == 0 || tokens.len() != 4 + (width * height * 3) as usize { bail!("invalid PPM pixel data"); }
    let bytes: Result<Vec<u8>> = tokens[4..].iter().map(|s| { let v: u32 = s.parse()?; if v > max { bail!("channel exceeds max") }; Ok(((v as f64 / max as f64 * 255.0).round() as u32).min(255) as u8) }).collect();
    let image = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, bytes?).ok_or_else(|| anyhow!("invalid PPM dimensions"))?;
    image.save(output)?; Ok((width, height))
}

pub fn read_memory(goal: &str) -> Vec<MemoryItem> {
    let path = state_root().join("memory/memory.json");
    let mut items: Vec<MemoryItem> = fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    let terms = terms(goal);
    items.sort_by_key(|item| std::cmp::Reverse(terms.iter().filter(|term| item.keywords.contains(term) || item.descriptor.to_lowercase().contains(*term)).count()));
    items.into_iter().filter(|i| matches!(i.kind.as_str(), "hack" | "pointer")).take(8).collect()
}
fn terms(text: &str) -> HashSet<String> { text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|s| s.len() > 2).take(12).map(str::to_owned).collect() }
fn append_memory(item: &MemoryItem) -> Result<()> {
    let global = state_root().join("memory/memory.json");
    let mut entries: Vec<MemoryItem> = fs::read(&global).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    entries.push(item.clone()); atomic_write(&global, &serde_json::to_vec_pretty(&entries)?)?;
    let run = state_root().join("memory/runs").join(&item.run_id).join("memory.json");
    atomic_write(&run, &serde_json::to_vec_pretty(&vec![item])?)
}

/// Turns the prose + ```rust fence reply into the DecisionOutput shape the rest
/// of the pipeline (artifacts, UI events, extract_and_apply) already understands.
fn parse_decision_text(text: &str) -> Result<DecisionOutput> {
    let code = extract_rust_block(text)
        .ok_or_else(|| anyhow!("reply contained no ```rust code block — respond with reasoning followed by ONE fenced block holding the complete program"))?;
    let reasoning: String = text[..text.find("```").unwrap_or(0)].trim().chars().take(1200).collect();
    Ok(DecisionOutput {
        reasoning,
        tool_calls: vec![
            ToolCall { name: "create_file".into(), arguments: json!({"path": "scene.rs", "content": code}) },
            ToolCall { name: "run_to_ppm".into(), arguments: json!({}) },
            ToolCall { name: "ppm_to_png".into(), arguments: json!({}) },
        ],
        self_critique: String::new(),
    })
}

/// Returns the contents of the last fenced code block, tolerating a missing
/// language tag; falls back to the whole reply if it is bare Rust source.
fn extract_rust_block(text: &str) -> Option<String> {
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        let after_fence = &rest[start + 3..];
        let body_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        let body = &after_fence[body_start..];
        let Some(end) = body.find("```") else { break };
        blocks.push(body[..end].trim().to_owned());
        rest = &body[end + 3..];
    }
    if let Some(block) = blocks.into_iter().filter(|b| b.contains("fn main")).last() {
        return Some(block);
    }
    if text.contains("fn main") && text.contains("use ray_tracing_challenge_rs") {
        return Some(text.trim().to_owned());
    }
    None
}
fn perception_schema() -> Value { json!({"type":"object","additionalProperties":false,"required":["verdict","critiques","suggestion_rank"],"properties":{"verdict":{"enum":["goal_achieved","partial","wrong"]},"critiques":{"type":"array","maxItems":8,"items":{"type":"string","maxLength":200}},"measurements":{"type":"array","items":{"type":"object","required":["label","action"],"properties":{"label":{"type":"string"},"action":{"type":"string"},"percent":{"type":"number"}}}},"suggestion_rank":{"type":"integer","minimum":0,"maximum":100}}}) }

fn kb() -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("kb");
    // scene_template.md comes first: it carries the exact import block and a
    // compile-tested program the model must start from.
    ["scene_template.md","primitives.md","groups.md","camera_and_canvas.md","composition_recipes.md"]
        .iter().filter_map(|name| fs::read_to_string(dir.join(name)).ok())
        .collect::<Vec<_>>().join("\n").chars().take(10000).collect()
}
fn decision_prompt(goal: &str, iteration: u32, max: u32, previous: Option<&str>, feedback: Option<&PerceptionOutput>, memory: &[MemoryItem], failure: Option<&str>) -> String {
    format!("You are the decision agent for a Rust ray-tracing composition loop. You only see text; do not claim to inspect images. Use only the exact API shown in the KNOWLEDGE BASE.\n\
RESPONSE FORMAT (no JSON):\n\
1. One short paragraph explaining what you are building or changing and why.\n\
2. Exactly ONE fenced code block starting with ```rust that contains the COMPLETE compilable program, from the use statements through the final println. Never elide code or write `// ...`.\n\
RULES:\n\
- Copy the `use ray_tracing_challenge_rs::...` import block from the scene template VERBATIM. There is no `prelude` module and no `ray_tracer` crate; inventing imports fails the build.\n\
- `set_transform` takes a Matrix by value: use `set_transform(translation(...))` for one transform, or `set_transform(&translation(...) * &scaling(...))` for a composition. Do not borrow a single transform and do not multiply unborrowed matrices.\n\
- On later iterations, output the full revised program with the visual feedback applied to the PREVIOUS CODE.\n\
- The program must render with a 400x400 camera, write the PPM with std::fs::write, and its FINAL println! must be exactly `Saved to media/images_ppm/shape_composer_scene.ppm` matching the written path.\n\
GOAL: {goal}\nITERATION: {iteration}/{max}\n{}PREVIOUS CODE:\n{}\nLAST VISUAL FEEDBACK:\n{}\nMEMORY HINTS:\n{}\nKNOWLEDGE BASE:\n{}",
        failure.map(|f| format!("FAILURE (your previous response failed; fix this exact problem):\n{f}\n")).unwrap_or_default(),
        previous.unwrap_or("(none)"), feedback.map(|x| serde_json::to_string(x).unwrap()).unwrap_or_else(|| "(none)".into()),
        memory.iter().map(|m| m.descriptor.as_str()).collect::<Vec<_>>().join("\n"), kb())
}
fn perception_prompt(goal: &str, iteration: u32, previous: Option<&PerceptionOutput>, memory: &[MemoryItem]) -> String {
    format!("You are the perception agent. You receive two PNG images in this exact order: FIRST is the current rendered scene to judge; SECOND is the visual reference image for the goal. Discuss only the FIRST image in your critique, never Rust or code. Use the SECOND image only as inspiration for composition, silhouette, pose, proportions, and dynamism; do not demand literal pixel-for-pixel copying. Goal: {goal}. Iteration: {iteration}. Previous critique: {}. Memory visual hints: {}. Return goal_achieved only if a human would call it done; then critiques must be empty and suggestion_rank 100. Otherwise give concise actionable visual-only critiques such as spatial gaps, scale percentages, camera framing, silhouette, lighting.",
        previous.map(|x| serde_json::to_string(x).unwrap()).unwrap_or_else(|| "(none)".into()), memory.iter().map(|m| m.descriptor.as_str()).collect::<Vec<_>>().join("; "))
}

pub async fn run(request: RunRequest) -> Result<RunSummary> {
    let started = std::time::Instant::now();
    let run_id = std::env::var("SHAPE_COMPOSER_RUN_ID")
        .unwrap_or_else(|_| format!("run-{}", &Uuid::new_v4().simple().to_string()[..8]));
    let run_dir = state_root().join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)?;
    atomic_write(&run_dir.join("goal.txt"), request.goal.as_bytes())?;
    let gateway = Gateway::from_env()?;
    let memory = read_memory(&request.goal);
    let reference_path = &request.reference_image;
    let reference_png = fs::read(reference_path)
        .with_context(|| format!("read visual reference image {}", reference_path.display()))?;
    let max = request.max_iterations.clamp(1, MAX_ITERATIONS);
    let mut previous_code = request.seed_prompt.clone();
    let mut feedback: Option<PerceptionOutput> = None;
    let mut final_png = None;
    let mut written = 0;
    let mut final_verdict = Verdict::Wrong;
    let mut completed_iterations = 0;
    eprintln!("[{run_id}] goal: {}", request.goal);
    for n in 1..=max {
        let version = version_name(n);
        eprintln!("[{run_id}] iteration {n}/{max}: asking decision agent");
        emit_event(&run_id, n, "DecisionStarted", json!({})).await;
        // One corrective hop per iteration: retry the decision once with the failure
        // report, then skip the iteration keeping the previous code (invariant §1.2 #7).
        let mut produced = None;
        let mut failure: Option<String> = None;
        for _hop in 0..2 {
            match produce_version(&gateway, &request, n, max, previous_code.as_deref(), feedback.as_ref(), &memory, failure.as_deref(), &run_dir, &run_id).await {
                Ok(result) => { produced = Some(result); break; }
                Err(err) => {
                    let message = format!("{err:#}");
                    eprintln!("[{run_id}] iteration {n} attempt failed: {message}");
                    emit_event(&run_id, n, "Error", json!({"error": message, "will_retry": failure.is_none()})).await;
                    failure = Some(message);
                }
            }
        }
        let Some((code, png)) = produced else {
            eprintln!("[{run_id}] iteration {n} skipped after corrective retry");
            continue;
        };
        eprintln!("[{run_id}] iteration {n}: rendered {}", png.display());
        emit_event(&run_id, n, "RenderCompleted", json!({"png": png.display().to_string()})).await;
        let perception_session = format!("{run_id}-perception");
        let perception = gateway.perceive(
            perception_prompt(&request.goal, n, feedback.as_ref(), &memory),
            &fs::read(&png)?,
            &reference_png,
            perception_schema(),
            &perception_session,
        ).await?;
        atomic_write(&run_dir.join(format!("{version}.feedback.json")), &serde_json::to_vec_pretty(&perception)?)?;
        if let Some(critique) = perception.critiques.iter().find(|c| c.contains('%') || c.contains("percent")) {
            let item = MemoryItem { id: Uuid::new_v4().to_string(), kind:"pointer".into(), keywords: terms(&request.goal).into_iter().collect(), descriptor:critique.clone(), value:json!({}), source:"perception".into(), run_id:run_id.clone(), created_at:now() };
            append_memory(&item)?; written += 1;
        }
        eprintln!("[{run_id}] iteration {n}: verdict {:?}, critiques: {:?}", perception.verdict, perception.critiques);
        final_png = Some(png.display().to_string()); final_verdict = perception.verdict.clone(); previous_code = Some(code); feedback = Some(perception);
        completed_iterations = n;
        emit_event(&run_id, n, "PerceptionCompleted", serde_json::to_value(feedback.as_ref()).unwrap_or_default()).await;
        if final_verdict == Verdict::GoalAchieved { break; }
    }
    let summary = RunSummary { run_id:run_id.clone(), goal:request.goal, mode:request.mode, iterations: completed_iterations, final_verdict, final_png, duration_s:started.elapsed().as_secs_f64(), memory_items_written:written };
    atomic_write(&run_dir.join("summary.json"), &serde_json::to_vec_pretty(&summary)?)?;
    emit_event(&run_id, summary.iterations, "RunDone", serde_json::to_value(&summary).unwrap_or_default()).await;
    Ok(summary)
}

#[allow(clippy::too_many_arguments)]
async fn produce_version(
    gateway: &Gateway, request: &RunRequest, n: u32, max: u32,
    previous_code: Option<&str>, feedback: Option<&PerceptionOutput>, memory: &[MemoryItem],
    failure: Option<&str>, run_dir: &Path, run_id: &str,
) -> Result<(String, PathBuf)> {
    let version = version_name(n);
    let prompt = decision_prompt(&request.goal, n, max, previous_code, feedback, memory, failure);
    let decision = gateway.decision(prompt, run_id).await?;
    atomic_write(&run_dir.join(format!("{version}.decision.json")), &serde_json::to_vec_pretty(&decision)?)?;
    emit_event(run_id, n, "DecisionCompleted", json!({"reasoning": decision.reasoning})).await;
    let code = extract_and_apply(run_dir, n, previous_code, &decision)?;
    let rs = run_dir.join(format!("{version}.rs"));
    atomic_write(&rs, code.as_bytes())?;
    if let Some(old) = previous_code { atomic_write(&run_dir.join(format!("{version}.diff.patch")), simple_diff(old, &code).as_bytes())?; }
    let ppm = render_scene(&rs, run_dir, n).await?;
    let png = run_dir.join(format!("{version}.png"));
    ppm_to_png(&ppm, &png)?;
    Ok((code, png))
}

fn extract_and_apply(run_dir: &Path, n: u32, previous: Option<&str>, output: &DecisionOutput) -> Result<String> {
    let mut code = previous.unwrap_or_default().to_owned();
    // Gemini structured output sometimes attaches an argument to the wrong element
    // of the tool_calls array (e.g. `content` lands on run_to_ppm), so read
    // `content` and `edits` from ANY call instead of trusting the tool name.
    for call in &output.tool_calls {
        if let Some(content) = call.arguments.get("content").and_then(Value::as_str).filter(|c| !c.trim().is_empty()) {
            code = strip_code_fences(content);
        }
        if let Some(edits) = call.arguments.get("edits").and_then(Value::as_array) {
            for edit in edits {
                let find = edit.get("find").and_then(Value::as_str).ok_or_else(|| anyhow!("edit find missing"))?;
                let replace = edit.get("replace").and_then(Value::as_str).ok_or_else(|| anyhow!("edit replace missing"))?;
                let replace_all = edit.get("replace_all").and_then(Value::as_bool).unwrap_or(false);
                let count = code.matches(find).count();
                if count == 0 { bail!("edit text not found in previous code: {find}"); }
                if count != 1 && !replace_all { bail!("edit must match exactly once (matched {count} times): {find}"); }
                code = if replace_all { code.replace(find, replace) } else { code.replacen(find, replace, 1) };
            }
        }
    }
    if code.trim().is_empty() {
        bail!("no Rust source found in any tool call — put the ENTIRE program in the `content` string of create_file");
    }
    let _ = (run_dir, n); Ok(code)
}

fn strip_code_fences(content: &str) -> String {
    let trimmed = content.trim();
    if let Some(inner) = trimmed.strip_prefix("```") {
        let inner = inner.strip_prefix("rust").unwrap_or(inner);
        return inner.strip_suffix("```").unwrap_or(inner).trim().to_owned();
    }
    trimmed.to_owned()
}
fn simple_diff(old: &str, new: &str) -> String { format!("--- previous\n+++ current\n-{}\n+{}\n", old, new) }

async fn render_scene(source: &Path, run_dir: &Path, iteration: u32) -> Result<PathBuf> {
    let bin = format!("shape_composer_{}", version_name(iteration));
    let target = workspace_root().join("src/bin").join(format!("{bin}.rs"));
    fs::copy(source, &target)?;
    let output = timeout(Duration::from_secs(180), Command::new("cargo").args(["run","--release","--bin",&bin]).current_dir(workspace_root()).stdout(Stdio::piped()).stderr(Stdio::piped()).output()).await.context("render timed out")??;
    let _ = fs::remove_file(&target);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Keep the tail: rustc prints the error summary last and the prompt has a budget.
        let tail: String = stderr.chars().skip(stderr.chars().count().saturating_sub(6000)).collect();
        bail!("scene compilation/render failed:\n{tail}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let saved = stdout.lines().find_map(|line| line.strip_prefix("Saved to ")).ok_or_else(|| anyhow!("scene must print `Saved to <ppm path>`"))?;
    let source_ppm = workspace_root().join(saved);
    let dest = run_dir.join(format!("{}.ppm", version_name(iteration)));
    fs::copy(source_ppm, &dest)?; Ok(dest)
}

/// Runs the minimal newline-delimited MCP server. This deliberately has no LLM access.
pub async fn serve_mcp(run_dir: PathBuf) -> Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        let request: Value = serde_json::from_str(&line)?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => Ok(json!({"protocolVersion":"2024-11-05","serverInfo":{"name":"shape-composer-mcp","version":"0.1.0"}})),
            "tools/list" => Ok(json!({"tools":[{"name":"create_file"},{"name":"modify_file"},{"name":"run_to_ppm"},{"name":"ppm_to_png"}]})),
            "tools/call" => mcp_call(&run_dir, request.get("params").unwrap_or(&Value::Null)).await,
            _ => Err(anyhow!("unknown method")),
        };
        let response = match result { Ok(v) => json!({"jsonrpc":"2.0","id":id,"result":v}), Err(e) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":e.to_string()}}) };
        stdout.write_all(serde_json::to_string(&response)?.as_bytes()).await?; stdout.write_all(b"\n").await?; stdout.flush().await?;
    }
    Ok(())
}
async fn mcp_call(root: &Path, params: &Value) -> Result<Value> {
    let name = params.get("name").and_then(Value::as_str).ok_or_else(|| anyhow!("tool name missing"))?;
    let args = params.get("arguments").unwrap_or(&Value::Null);
    match name {
        "create_file" => { let path = safe_resolve(root, args["path"].as_str().ok_or_else(|| anyhow!("path missing"))?)?; if path.exists() { bail!("file already exists") }; let content=args["content"].as_str().ok_or_else(|| anyhow!("content missing"))?; atomic_write(&path, content.as_bytes())?; Ok(json!({"ok":true,"path":args["path"],"size_bytes":content.len()})) }
        "modify_file" => { let path=safe_resolve(root,args["path"].as_str().ok_or_else(|| anyhow!("path missing"))?)?; let mut content=fs::read_to_string(&path)?; let mut replacements=Vec::new(); for edit in args["edits"].as_array().ok_or_else(|| anyhow!("edits missing"))? { let find=edit["find"].as_str().ok_or_else(|| anyhow!("find missing"))?; let count=content.matches(find).count(); if count != 1 && !edit["replace_all"].as_bool().unwrap_or(false) { bail!("find must match exactly once") }; content=content.replace(find,edit["replace"].as_str().ok_or_else(|| anyhow!("replace missing"))?); replacements.push(count); } atomic_write(&path,content.as_bytes())?; Ok(json!({"ok":true,"replacements":replacements})) }
        "ppm_to_png" => { let ppm=safe_resolve(root,args["ppm_path"].as_str().ok_or_else(|| anyhow!("ppm_path missing"))?)?; let png=args.get("png_path").and_then(Value::as_str).map(|p| safe_resolve(root,p)).transpose()?.unwrap_or_else(|| ppm.with_extension("png")); let (width,height)=ppm_to_png(&ppm,&png)?; Ok(json!({"ok":true,"png_path":png,"width":width,"height":height})) }
        "run_to_ppm" => Err(anyhow!("run_to_ppm is orchestrator-owned; use the runner")),
        _ => bail!("unknown tool {name}"),
    }
}
