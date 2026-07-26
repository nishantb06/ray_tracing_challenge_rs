# Shape Composer Agent — Vision-Driven Iterative Scene Composition

> A self-contained Rust agent that dreams up expressive scenes out of the ray
> tracer's basic primitives (cube, cylinder, sphere, cone, plane, triangle,
> group, CSG), renders them to PPM, converts to PNG, and **iterates** by
> showing the PNG to a vision-capable *perception* agent (Gemini) until the
> composition matches the user's goal — with the decision agent translating
> visual critique into Rust code edits.

This plan takes inspiration from `plans/agent-spec.md` (the Ollive growing-DAG
architecture) but is **deliberately much smaller**: two agents (decision +
perception), one MCP server with four tools, a simple working-memory store, and
a UI tab stitched into the existing `live_server` Vite SPA. Everything is Rust;
the only external dependency is the LLM gateway on `:8108` (which already routes
to Gemini).

---

## 0. How to read this document

- §1 — the big picture: one iteration loop, two agents, the data that flows.
- §2 — the four-file Rust crate layout and its public types.
- §3 — the MCP server: exactly four tools, with JSON-schema and return shapes.
- §4 — the decision agent: prompt contract, output JSON, edit grammar.
- §5 — the perception agent: the **only** image-touching component; Gemini
  vision call, feedback JSON, "goal achieved" verdict.
- §6 — process memory: a tiny JSON file of "hacks and pointers" that survives
  across runs and is injected into both agents' prompts.
- §7 — the iteration runner: the loop, autosave of `v1.rs, v2.rs, …`,
  termination in auto / HIL modes, run state on disk.
- §8 — frontend integration into `live_server`: new `/agent` WebSocket, new
  SPA tab, "chain of images + feedback" panel, nomenclature.
- §9 — knowledge base: small markdown docs about the ray tracer, consumed by
  the decision agent's prompt.
- §10 — sample "beautiful prompts" the system can run out of the box.
- §11 — full build & verification recipe (cargo steps, ports, env vars).
- §12 — phase breakdown so you can ship in slices.
- §13 — risks, scope cuts, and out-of-scope items.

---

## 1. High-Level Architecture

### 1.1 One iteration, two agents

The agent executes a **single perceive→decide→act→render→perceive** loop —
NOT the growing DAG from `agent-spec.md`. The DAG architecture is overkill for
"keep editing one `.rs` file until the picture matches the goal"; we keep only
its best ideas: typed memory, MCP tools, structured LLM output, atomic writes,
and separation of perception (vision) from decision (code).

```
                    ┌──────────────────┐
                    │  user goal (text) │  e.g. "a human figure made of
                    │                   │         boxes and cylinders"
                    └─────────┬────────┘
                              ▼
                    ┌──────────────────┐
                    │ process memory   │  read ONCE per run (vs §1.1 of
                    │ read(query)      │  agent-spec), injected into every
                    └─────────┬────────┘  decision & perception prompt
                              ▼
            ┌─────────────────────────────────┐
            │  iteration  N  (= starts at 1)   │
            │                                  │
            │  1. decision agent (LLM, text)   │── 4 MCP tools available ──┐
            │     • create_file vN.rs         │                            │
            │     • modify_file vN.rs         │                            │
            │     • run_to_ppm    vN.rs        │                            │
            │     • ppm_to_png    vN.ppm       │                            │
            │  2. sandbox: cargo run --bin vN │── produces vN.ppm, vN.png  │
            │  3. perception agent (Gemini)    │                            │
            │     • inspect vN.png             │   vision → JSON feedback   │
            │     • emit verdict + critique   │                            │
            │  4. process memory.append        │                            │
            │     useful hacks discovered     │                            │
            │  5. UI: push (vN.png, feedback,  │                            │
            │     code diff) to frontend      │                            │
            │                                  │                            │
            │  terminate?  ──────────────────►│                            │
            │    • perception.verdict ==       │                            │
            │      "goal_achieved"  OR         │                            │
            │    • N >= MAX_ITER (default 25)  │                            │
            │    • HIL mode → user clicks      │                            │
            │      "Approve" / "Stop"          │                            │
            └──────┬───────────────────────────┘                            │
                   │ N += 1                                                  │
                   ▼                                                          │
            ┌────────────────────────────────────┐                           │
            │ next decision agent prompt         │  inputs:                  │
            │  • USER_GOAL                       │   - goal                 │
            │  • last_code (vN.rs)               │   - last code            │
            │  • last_feedback (perception)      │   - last feedback        │
            │  • memory_hits (hacks & pointers)  │   - memory hits          │
            │  • KB excerpts (ray tracer API)    │   - KB excerpts          │
            └────────────────────────────────────┘                           │
                                                                             │
                                              MCP server (Rust stdio) ◄──────┘
                                              • create_file
                                              • modify_file
                                              • run_to_ppm
                                              • ppm_to_png
```

### 1.2 Invariants the architecture MUST preserve

1. **Exactly one `.rs` file is being iterated on** per run, under
   `state/runs/<run_id>/vN.rs`. The decision agent never edits multiple files.
2. **Perception is the only agent that touches PNG files.** It reads the PNG
   bytes produced by the `ppm_to_png` MCP tool and sends them to Gemini's
   vision endpoint via the gateway. The decision agent never sees pixels.
3. **Process memory is read ONCE per run** (mirror of agent-spec §1.1 #3) and
   the same `Vec<MemoryItem>` is reused as prompts are rendered across
   iterations.
4. **Each iteration produces a deterministic set of artifacts on disk**:
   `vN.rs`, `vN.ppm`, `vN.png`, `vN.feedback.json`, `vN.code_diff.patch`. This
   is what the UI chains as "iterations".
5. **Version nomenclature is uniform**: zero-padded `v01`, `v02`, …, `v25`.
   The agent binary is invoked as `cargo run --bin shape_composer_v01` (the
   runner copies `v01.rs` into `src/bin/shape_composer_v01.rs` before invoking
   cargo, and deletes it after — details in §7.4).
6. **Auto vs. HIL is a run-level flag**. In auto mode, the loop stops only on
   `verdict == goal_achieved` or `MAX_ITER`. In HIL mode, the runner pauses
   after each perception step and waits for a UI signal
   (`approve | edit_feedback | stop`).
7. **The decision agent always speaks JSON**, validated against a strict
   schema (see §4.3). On validation failure, the iteration is retried once;
   a second failure marks the iteration as `failed` and the loop continues
   with the previous code (so the UI shows a "stuck iteration" without
   crashing the whole run).
8. **The MCP server is a child subprocess** speaking JSON-RPC over stdio,
   spawned once per run (NOT per iteration — it lives for the whole run so
   that file paths and the cargo target dir stay warm). See §3.4.

### 1.3 Process boundaries

| Process | Lifetime | Responsibility |
|---|---|---|
| `shape_composer` (agent runner) | per run | orchestrates the loop, owns memory, talks to gateway, spawns MCP |
| `mcp_server` (Rust stdio child) | per run | the four file/render tools; never touches the LLM |
| `live_server` (existing Axum) | long-lived | serves the SPA, exposes `/agent` WS, proxies to gateway if needed |
| LLM gateway on `:8108` | external | chat + embed; we use it as-is (see agent-spec §3) |
| browser SPA | per tab | the Agent tab subscribes to `/agent` WS for a given run id |

The agent runner **does not** embed a web server. It connects to
`live_server`'s `/agent` ingest endpoint over HTTP (a long-poll or SSE) or
opens a reverse WebSocket client to push iteration events. We choose the
simpler **HTTP POST ingest + SSE fanout** model (§8.2) so the agent runner
needs only an HTTP client.

---

## 2. Crate layout & public types

### 2.1 Directory structure (new crate, top-level workspace member)

```
ray_tracing_challenge_rs/
├── Cargo.toml                  (workspace already covers live_server, live_viewer)
│   [workspace] members = [..., "shape_composer"]
│
├── shape_composer/             ★ NEW crate
│   ├── Cargo.toml
│   ├── prompts/
│   │   ├── decision.md         # decision agent system prompt
│   │   ├── perception.md       # perception agent system prompt
│   │   ├── goal_dreamer.md     # the "prompt dreamer" system prompt
│   │   └── memory_classify.md  # classifier prompt for process-memory writes
│   ├── kb/                     # knowledge base markdown (consumed by decision)
│   │   ├── README.md
│   │   ├── primitives.md
│   │   ├── transforms.md
│   │   ├── camera_and_canvas.md
│   │   ├── materials_and_patterns.md
│   │   ├── lights.md
│   │   ├── groups_and_csg.md
│   │   └── composition_recipes.md
│   └── src/
│       ├── main.rs             # CLI: entry point, arg parsing
│       ├── schemas.rs          # all typed contracts (mirror of agent-spec §2.1)
│       ├── runner.rs           # the iteration loop
│       ├── decision.rs         # decision agent (LLM call + JSON parse)
│       ├── perception.rs       # perception agent (Gemini vision via gateway)
│       ├── memory.rs           # process memory (simple JSON list + keyword recall)
│       ├── kb.rs               # KB loader (reads kb/*.md, returns excerpts)
│       ├── prompt.rs           # prompt renderer
│       ├── gw_client.rs        # HTTP client to :8108 (chat + embed)
│       ├── ui_ingest.rs        # HTTP POST → live_server /agent/ingest
│       ├── mcp_runner.rs       # stdio MCP client (initialize, tools/list, tools/call)
│       └── sandbox.rs          # cargo subprocess + PPM/PNG file ops helpers
│
└── (existing) live_server/
    ├── src/
    │   ├── main.rs             # add /agent routes (UI ingest + SSE fanout)
    │   ├── protocol.rs         # add Agent* message types
    │   └── agent_state.rs      ★ NEW: in-memory per-run event bus
    └── web/
        └── src/
            ├── app.ts          # add "Agent" tab/view
            ├── agent.ts        ★ NEW: agent tab logic + SSE client
            ├── agent.css       ★ NEW: styling for the iteration gallery
            └── protocol.ts     # extend with Agent* types
```

### 2.2 `shape_composer/Cargo.toml`

```toml
[package]
name = "shape_composer"
version = "0.1.0"
edition = "2024"

[dependencies]
ray_tracing_challenge_rs = { path = ".." }   # reuse the engine's types in KB/help
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "blocking", "rustls-tls"], default-features = false }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "sync", "fs", "io-util", "time"] }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
base64 = "0.22"
image = "0.25"   # ONLY for cheap in-process PPM→RGBA→PNG sanity checks; the
                 # actual PPM→PNG used by the agent is the MCP tool (cargo run
                 # --bin ppm_to_png) so the decision agent learns the engine's
                 # idioms. (See §3.3.)
sha2 = "0.10"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
tempfile = "3"
```

> Edition is **2024** to match the workspace. We use `tokio::process::Command`
> to spawn both the MCP server (a child cargo binary) and the
> `cargo run --bin` calls. `reqwest` is the only HTTP client.

### 2.3 `schemas.rs` — the typed contracts

Mirror of agent-spec §2.1, trimmed to what we need.

```rust
pub fn new_run_id() -> String { format!("run-{}", &uuid::Uuid::new_v4().to_string()[..8]) }

pub fn version_name(n: u32) -> String { format!("v{:02}", n) }   // "v01".."v25"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode { Auto, Hil }   // serialized lowercase via serde rename_all

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
    pub goal: String,                 // user-stated goal
    pub mode: Mode,
    pub max_iterations: u32,          // default 25, cap 60
    pub model_profile: Option<String>, // null → gemini default
    pub seed_prompt: Option<String>,  // optional hand-written starter code
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryKind { Fact, Hack, Pointer, Preference, Scratchpad }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub kind: MemoryKind,
    pub keywords: Vec<String>,
    pub descriptor: String,           // ≤200 chars, what the agent reads
    pub value: serde_json::Value,     // free-form; never null
    pub source: String,              // "decision" | "perception" | "user"
    pub run_id: String,
    pub created_at: String,          // ISO-8601 UTC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutput {
    pub reasoning: String,           // short chain-of-thought, NOT shown to perception
    pub tool_calls: Vec<ToolCall>,    // ordered list — applied sequentially
    pub self_critique: String,        // "what could go wrong with this edit"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,                 // "create_file" | "modify_file" | "run_to_ppm" | "ppm_to_png"
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionOutput {
    pub verdict: Verdict,             // GoalAchieved | Partial | Wrong
    pub critiques: Vec<String>,        // human-readable list, e.g. "torso and head not close enough"
    pub measurements: Vec<Measurement>, // optional numerical hints
    pub suggestion_rank: u8,           // 0..=100 — confidence the next edit will help
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict { GoalAchieved, Partial, Wrong }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub label: String,                // "upper_cube_width"
    pub action: String,               // "reduce"
    pub percent: Option<f32>,         // "by 25 percent" → 25
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationEvent {           // pushed to UI per iteration step
    pub run_id: String,
    pub iteration: u32,
    pub phase: Phase,
    pub payload: serde_json::Value,
    pub ts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    DecisionStarted, DecisionCompleted,
    ToolCallStarted, ToolCallCompleted,
    RenderStarted, RenderCompleted,
    PerceptionStarted, PerceptionCompleted,
    MemoryUpdated,
    HILAwaitingApproval, HILApproved, HILStopped,
    IterationDone,
    RunDone,
    Error,
}
```

### 2.4 The 4 MCP tools (visibility table for the decision agent)

| Tool | Args | Returns | Side effects |
|---|---|---|---|
| `create_file` | `{path, content}` | `{ok, path, size_bytes}` | writes (or refuses if exists) under `state/runs/<run_id>/` |
| `modify_file` | `{path, edits: [{find, replace, replace_all?}]}` | `{ok, path, replacements: [int]}` | apply ordered edits; errors if a `find` doesn't match (count==0) or matches >1 without `replace_all` |
| `run_to_ppm` | `{bin_name}` | `{ok, ppm_path, stdout_tail, stderr_tail, elapsed_ms, timed_out}` | `cargo run --release --bin <bin_name>`; the runner guarantees the bin was materialized first via `create_file`/`modify_file` |
| `ppm_to_png` | `{ppm_path, png_path?}` | `{ok, png_path, width, height}` | shells out to the existing `ppm_to_png` binary or — for speed — does the conversion in-process (see §3.3) |

The decision agent's prompt explicitly forbids calling `run_to_ppm` before
a successful `create_file` or `modify_file` in the same iteration.

---

## 3. The MCP Server (Rust, stdio, 4 tools)

### 3.1 Crate layout for the MCP server

We implement the MCP server **as a binary inside `shape_composer`** (not a
separate crate) so that it shares the `schemas` types and the cargo target
dir:

```rust
// shape_composer/src/bin/mcp_server.rs
use shape_composer::mcp_server::serve;
fn main() { serve() }
```

This binary is spawned by the agent runner as a stdio child (see §3.4). It
is **stateless between runs** — all state lives in `state/runs/<run_id>/`.

### 3.2 Protocol: minimal MCP over stdio JSON-RPC

We implement three methods only:

- `initialize` → returns `{protocolVersion: "2024-11-05", serverInfo: {name: "shape-composer-mcp", version: "0.1.0"}}`
- `tools/list` → returns the four tool descriptors (hand-written; we do NOT
  need a general registry for four tools)
- `tools/call` → dispatches by `name` to one of four `async fn`s

Frames are newline-delimited JSON (the simple `Content-Length: N` headers
are NOT used — we follow the "line-delimited JSON-RPC" subset that many MCP
clients accept, which is what `rmcp`/`mcp-rust` use by default in their
stdio transport). Each request and response is one line.

> If the chosen MCP client crate mandates the framed `Content-Length`
> variant, switch to that — it's a transport-only decision and the tool
> implementations are unchanged.

### 3.3 Tool implementations

#### 3.3.1 `create_file`
```rust
async fn create_file(args: CreateFileArgs, run_dir: &Path) -> Result<Value> {
    // run_dir is passed to the MCP child as argv[1]; tool args path is resolved
    // under run_dir — refuse any absolute or ".."-containing path.
    let path = safe_resolve(run_dir, &args.path)?;
    if path.exists() { bail!("file already exists: {}", args.path); }
    if let Some(p) = path.parent() { fs::create_dir_all(p).await?; }
    fs::write(&path, args.content.as_bytes()).await?;
    Ok(json!({ "ok": true, "path": args.path, "size_bytes": args.content.len() }))
}
```

#### 3.3.2 `modify_file`
Apply an ordered list of `{find, replace, replace_all}` edits. Each edit
must match exactly once, unless `replace_all == true`. Returns the number of
replacements per edit. Atomicity: write to `path.tmp` then rename (mirror of
agent-spec §2.13).

#### 3.3.3 `run_to_ppm`
Two implementations available; pick one and document it:

- **Option A (recommended for v1): in-process render.** The MCP server is
  part of the `shape_composer` crate, which depends on
  `ray_tracing_challenge_rs`. It reads the bin source from
  `state/runs/<run_id>/<bin_name>.rs`, **`lib::eval`**-style in-processes it
  is NOT feasible since the bins are full programs — so instead we copy the
  file into `src/bin/<bin_name>.rs` (the actual workspace src/bin), invoke
  `cargo run --release --bin <bin_name>`, wait for the PPM to land in the
  cwd the bin writes to, then read the PPM path from the bin's stdout
  (convention: every bin prints `Saved to media/images_ppm/<name>.ppm`).
  We then move that PPM into `state/runs/<run_id>/<bin_name>.ppm`.

- **Option B: shell out fully.** `cargo run --release --bin <bin_name>` and
  parse the printed "Saved to" line. Identical to A except we never read the
  source from disk inside the tool — we trust the decision agent called
  `create_file`/`modify_file` first (the runner also asserts this).

Both options reuse the existing `media/images_ppm/` convention. The runner
must **ensure** the materialized `src/bin/<bin_name>.rs` is deleted in a
`finally`/`Drop` so the workspace isn't polluted. We pick **Option B** for
v1 (simpler) and revisit Option A if cargo rebuilds become a bottleneck.

> **Important**: the bin name passed to `run_to_ppm` must match the file the
> decision agent created via `create_file`/`modify_file`. The runner
> enforces: in any single iteration the agent can only render a bin whose
> source file it just touched. This stops the agent from running arbitrary
> existing bins.

#### 3.3.4 `ppm_to_png`
Reuse the existing `src/bin/ppm_to_png.rs` logic — since it depends only on
the `image` crate, and `shape_composer` already pulls `image`, we **copy
that logic into a `shape_composer::ppm_to_png` module** and call it directly
(no subprocess). Returns `{ok, png_path, width, height}` by writing the
PNG next to the PPM (or at `png_path` if given). This is much faster than
spawning `cargo run --bin ppm_to_png` for every iteration.

> We deliberately keep the *agent's* path simple (one in-process call) while
> leaving the standalone `ppm_to_png` bin untouched for human use.

### 3.4 MCP child lifecycle

The agent runner:

1. Spawns `cargo run --release --bin mcp_server -- <run_dir>` as a
   `tokio::process::Child` with stdin/stdout piped, stderr inherited.
2. Sends `initialize`, awaits response, sends `tools/list` (for sanity).
3. The child lives for the whole run; the agent calls `tools/call` once per
   decision-agent tool invocation, awaiting the response before the next
   chat call (mirror of agent-spec §2.6's single-hop discipline — but here
   the *agent* drives the multi-turn tool loop, not a separate `mcp_runner`;
   see §4.4 for why).
4. On run end (normal or panic), the runner drops the child → it is killed.

> We chose to let the **decision agent's** LLM responses include a list of
> `tool_calls` (rather than driving a multi-turn tool loop). This is simpler
> than agent-spec §2.6 and is fine because our four tools are deterministic
> and ordered. The runner applies each `tool_call` in order, feeding the
> previous tool's JSON result into the decision agent's next prompt only if
> a tool errors (otherwise we proceed straight to render).

---

## 4. The Decision Agent

### 4.1 Role

Given: (a) the user's goal, (b) the previous version's source code (or
a seed), (c) the perception agent's last feedback (if any), (d) KB
excerpts, (e) process-memory hits — emit a JSON `DecisionOutput`
containing a list of **MCP tool calls** that, when applied, will produce
the next version of the `.rs` file and render it.

The decision agent **never** sees PNG bytes. It only reads text feedback
from perception.

### 4.2 Inputs to the prompt (`prompt.rs::render_decision`)

Concatenate in this order (mirror of agent-spec §2.4.4 but simpler):

1. `prompts/decision.md` (system prompt, stripped).
2. `GOAL: <user goal>` (always).
3. `MODE: auto | hil` (so the agent knows whether to be cautious).
4. `ITERATION: <N>` and `MAX_ITERATIONS: <M>`.
5. `PREVIOUS CODE (v{N-1}.rs)` — wrapped in a fenced rust block, max 20000
   chars. For iteration 1, "PREVIOUS CODE: none" or the seed prompt.
6. `LAST FEEDBACK (perception v{N-1}):` — the JSON of `PerceptionOutput`
   from the previous iteration, or "none" for iteration 1.
7. `MEMORY HITS (<n>):` — formatted as `[<kind>] <descriptor>\n  preview`
   (max 8 hits, max 2000 chars total).
8. `KNOWLEDGE BASE EXCERPTS:` — see §9.3 for selection; max 8 KB sections,
   max 6000 chars total. Always include `primitives.md`,
   `camera_and_canvas.md`, and one recipe from `composition_recipes.md`.
9. `AVAILABLE MCP TOOLS:` — the four tool descriptors (Schema + short
   description), so the model knows the exact call syntax.
10. `OUTPUT FORMAT:` — a strict JSON schema reminder (see §4.3).

### 4.3 Output schema (strict `json_schema` over the gateway)

```jsonc
{
  "type": "object",
  "additionalProperties": false,
  "required": ["reasoning", "tool_calls", "self_critique"],
  "properties": {
    "reasoning": { "type": "string", "maxLength": 600 },
    "tool_calls": {
      "type": "array",
      "minItems": 1,
      "maxItems": 6,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["name", "arguments"],
        "properties": {
          "name": { "enum": ["create_file", "modify_file", "run_to_ppm", "ppm_to_png"] },
          "arguments": { "type": "object" }
        }
      }
    },
    "self_critique": { "type": "string", "maxLength": 400 }
  }
}
```

The runner asks the gateway with `response_format: { type: "json_schema",
name: "DecisionOutput", strict: true, schema: <above> }` (mirror of
agent-spec §3.6's structured-output example). On a 503 (validation failure),
we retry once with the gateway's built-in corrective retry; a second
failure yields a `Phase::Error` iteration and we proceed with the previous
code (invariant §1.2 #7).

### 4.4 Why a flat tool_calls list, not a multi-turn loop?

agent-spec §2.6 spawns an MCP server and lets the model drive a
`chat → tool_call → tool_result → chat → …` loop with up to 6 hops. For 4
deterministic, ordered file ops, that loop is overkill and costs extra
round-trips. Instead:

- The model emits the full ordered `tool_calls` list in one structured
  response.
- The runner applies them in order via the MCP child.
- If a tool errors (file exists, find doesn't match, render fails), the
  runner sends **one** corrective chat call to the decision agent that
  includes the failing tool's result + asks for a fixed `tool_calls` list.
  At most one such corrective hop per iteration. If it still fails, the
  iteration is marked `failed` and we move on.

This keeps the chat-call count at roughly **1 decision call + 1 perception
call per iteration** (plus an occasional corrective call), which is well
within Gemini free-tier quotas.

### 4.5 Edit grammar the decision agent should follow

From `prompts/decision.md` (excerpt):

> - The file you are editing is `state/runs/<run_id>/v<N>.rs`. The first
>   iteration uses `create_file`. Every subsequent iteration uses
>   `modify_file` with **small targeted edits** (preferred) over rewriting
>   the whole file (allowed when the perception agent says "completely
>   wrong composition").
> - Always end your `tool_calls` with a `run_to_ppm` followed by
>   `ppm_to_png` so that `v<N>.png` exists for the perception agent.
> - The file must be self-contained (no external `use` other than
>   `ray_tracing_challenge_rs::*` and `std`).
> - It must print exactly: `Saved to <ppm_path>` on its last line, so the
>   `run_to_ppm` tool can locate the output.

### 4.6 Files the decision agent is forbidden from touching

- Anything outside `state/runs/<run_id>/`.
- The KB `kb/*.md` files.
- The prompts `prompts/*.md`.
- The process memory files (those are append-only from `memory.rs`, see §6).

The MCP `safe_resolve` enforces this at the filesystem layer.

---

## 5. The Perception Agent

### 5.1 Role

The only component allowed to read PNG bytes. It receives the rendered PNG
plus the user's goal, and emits a `PerceptionOutput`: a verdict
(goal_achieved / partial / wrong), a list of human-readable critiques
that the decision agent will read on the next iteration, and optional
numerical "make X smaller by Y%" hints.

### 5.2 Gemini vision via the gateway

The `live_server`'s gateway on `:8108` already supports chat with image
attachments (we verify this in §11.6 step 3; if the gateway does not yet
support image parts, we add a tiny pass-through route in `live_server`
that forwards to Gemini's `generateContent` with `inlineData` parts —
see §5.5 for the fallback).

Perception's chat request to the gateway:

```jsonc
POST /v1/chat
{
  "messages": [
    { "role": "user",
      "content": [
        { "type": "text", "text": "<rendered perception prompt>" },
        { "type": "image_url",
          "image_url": { "url": "data:image/png;base64,<base64 of v<N>.png>" } }
      ]
    }
  ],
  "provider": "gemini",
  "agent": "perception",
  "max_tokens": 1200,
  "temperature": 0.1,
  "response_format": {
    "type": "json_schema", "name": "PerceptionOutput", "strict": true,
    "schema": { /* §5.4 */ }
  }
}
```

### 5.3 Inputs to the perception prompt (`prompt.rs::render_perception`)

1. `prompts/perception.md` (system prompt — see §5.6 for the full text).
2. `GOAL: <user goal>`.
3. `ITERATION: <N>`.
4. `PREVIOUS CRITIQUES (v{N-1}):` — the last perception's critiques, so it
   doesn't repeat itself.
5. The PNG attached as the second content part (see §5.2).

### 5.4 Output schema (strict)

```jsonc
{
  "type": "object",
  "additionalProperties": false,
  "required": ["verdict", "critiques", "suggestion_rank"],
  "properties": {
    "verdict": { "enum": ["goal_achieved", "partial", "wrong"] },
    "critiques": { "type": "array", "minItems": 1, "maxItems": 8,
                   "items": { "type": "string", "maxLength": 200 } },
    "measurements": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["label", "action"],
        "properties": {
          "label": { "type": "string" },
          "action": { "enum": ["reduce", "grow", "move_left", "move_right",
                                "move_up", "move_down", "rotate"] },
          "percent": { "type": "number" }
        }
      }
    },
    "suggestion_rank": { "type": "integer", "minimum": 0, "maximum": 100 }
  }
}
```

### 5.5 Gateway image-support fallback

If `:8108/v1/chat` rejects `image_url` content parts (some provider
adapters only accept text), `perception.rs` falls back to calling
Gemini's REST API **directly** via `gw_client::gemini_vision_fallback`:

```
POST https://generativelanguage.googleapis.com/v1beta/models/
     gemini-2.5-flash:generateContent?key=<GEMINI_API_KEY>
{
  "contents": [{ "parts": [
    { "text": "<rendered perception prompt>" },
    { "inline_data": { "mime_type": "image/png",
                       "data": "<base64 png>" } }
  ]}],
  "generationConfig": { "responseMimeType": "application/json",
    "responseSchema": { /* §5.4 as a Gemini schema */ } }
}
```

We prefer the gateway path; the fallback is a defensive switch toggled by
`perception_use_direct_gemini` in the run config or an env var. The user
chose "through existing LLM gateway on :8108" as the default, so direct
Gemini is only a guarantee of last resort.

### 5.6 `prompts/perception.md` — full text (draft)

```md
You are a **perception agent** for a Rust ray-tracing composition loop.
You receive:
- a) the user's GOAL (a description of the scene the human wants),
- b) the iteration number,
- c) the critiques you gave on the previous iteration (so you don't repeat),
- d) one PNG image — the latest render of a Rust scene assembled from basic
     primitives: sphere, cube, cylinder, cone, plane, triangle, group, CSG.

You do NOT write or read Rust code. You only **look** at the image and answer:

1. `verdict`: one of `goal_achieved`, `partial`, `wrong`.
   - `goal_achieved` ONLY when the image matches the GOAL closely enough
     that the human would call it done.
   - `partial` when the composition is recognizably on the right track
     but specific parts are wrong.
   - `wrong` when the composition is fundamentally off (wrong subject,
     broken limbs, missing major parts, garbage render).

2. `critiques`: 1–8 short, concrete, **visual-only** sentences. Examples:
   - "the torso and the head are not close enough"
   - "the upper cube is too wide; make it narrower"
   - "the left arm cylinder is floating away from the shoulder"
   - "shadows are completely black; the scene is under-lit"
   Each critique ≤200 chars. Reference visible parts, NOT code.

3. `measurements` (optional): when a critique can be expressed as a
   proportional change, encode it. `percent` is the magnitude of the
   change you'd suggest (e.g. "make the upper cube smaller by 25 percent"
   → `{label: "upper cube", action: "reduce", percent: 25}`).

4. `suggestion_rank`: 0..=100 — how confident you are that acting on these
   critiques will move the next render closer to the goal.

Be terse. Do NOT speculate about the Rust code. Do NOT describe what you
cannot see. If the image is blank, all-black, or visibly a render failure,
return `verdict: "wrong"`, one critique saying "render failed", and
`suggestion_rank: 0`.
```

### 5.7 Feedback rules perception must follow

- Never mention line numbers, function names, or Rust syntax.
- Always phrase critiques visually so the decision agent has to map them
  to code (this side-steps the perception agent hallucinating APIs).
- If `verdict == goal_achieved`, `critiques` MUST be empty and
  `suggestion_rank` MUST be `100`. This is the termination signal in auto
  mode.

---

## 6. Process Memory

### 6.1 Role

Unlike agent-spec §2.8 (which embeds via FAISS), our process memory is a
**simple append-only JSON list with keyword-based recall**. We have maybe
tens to low-hundreds of items per run — vector search would be overkill.
Hacks and pointers from both the decision and perception agents are stored
here so future iterations (and future runs) converge faster.

### 6.2 File layout

```
state/
└── memory/
    ├── memory.json               # global, cross-run, append-only Vec<MemoryItem>
    └── runs/
        └── <run_id>/
            └── memory.json         # per-run subset (only items this run wrote)
```

Cross-run memory is what makes the agent "iterate faster next time": if a
prior run learned "cone angles need π/4 not π/2 for a believable forearm",
the next run with a similar goal should surface that hit.

### 6.3 Write surfaces (mirror of agent-spec §2.8, simplified)

1. `record_hack(descriptor, value, source, run_id)` — direct write, kind =
   `Hack` or `Pointer`. Used at the end of each iteration when the decision
   agent's `self_critique` mentions a reusable insight. The runner calls a
   small "memory extractor" LLM call (or a heuristic; see §6.5) to decide
   if the iteration produced anything worth remembering.
2. `record_preference(text, run_id)` — when the user types "approve but
   make faces less shiny" in HIL mode.

We do NOT implement the LLM classifier of agent-spec §2.8's `remember()`.
Kinds are assigned deterministically by the writer; no embedding is
stored. Cold-start items (none) and a deterministic tokenizer for keyword
recall are enough.

### 6.4 Read API

```rust
pub fn read(query: &str, top_k: usize) -> Vec<MemoryItem> {
    // tokenize query → Vec<String> (lowercase, split on non-alphanumeric,
    // drop stopwords, cap 12 tokens).
    // score each item by token-overlap count over (descriptor ∪ keywords).
    // return top_k items by score (ties broken by created_at desc).
}
```

This is the keyword-fallback path of agent-spec §2.8 promoted to primary.
Tuned for ≤5k items; if we ever exceed that, swap in a `usearch` or `hnsm`
index without changing the read API (the write API already stores the value
needed to embed later).

### 6.5 What gets remembered (heuristics, no extra LLM call in v1)

A MemoryItem is written at the end of iteration N if **any** of:

- The perception agent's critiques mention a quantitative measurement
  (e.g. "make the upper cube smaller by 25 percent") → record a `Pointer`
  with descriptor "goal <X>: <critique>".
- The decision agent's `self_critique` contains words like "next time",
  "remember", "always", "never" → record a `Hack`.
- The iteration's verdict improved from `Wrong → Partial` or
  `Partial → GoalAchieved` → record the last critique that was just
  resolved as a `Pointer` (descriptor: "X was fixed by Y").

This keeps memory write-rate ≤1 item per iteration, preventing bloat.

### 6.6 Memory consumption in prompts

Both decision and perception prompts include a `MEMORY HITS` block (see
§4.2 #7 and §5.3). For the decision agent we surface `Hack` and `Pointer`
kinds; for the perception agent we surface only `Pointer` (visual cues).

---

## 7. The Iteration Runner

### 7.1 `runner.rs::run` — top-level pseudocode

```rust
pub async fn run(req: RunRequest) -> Result<RunSummary> {
    let run_id = new_run_id();
    let run_dir = state_dir().join("runs").join(&run_id);
    fs::create_dir_all(&run_dir).await?;

    // 1. Spawn MCP child (lives for the whole run)
    let mut mcp = McpChild::spawn(&run_dir).await?;

    // 2. Read memory ONCE (invariant §1.2 #3)
    let mem = memory::load_all();
    let memory_hits = memory::read(&req.goal, 8);

    // 3. Open UI ingest sink (HTTP POST client to live_server /agent/ingest)
    let ui = UiIngest::start(&run_id, req.mode.clone());

    // 4. Optional seed prompt → write v01.rs directly
    let seed = req.seed_prompt.unwrap_or_else(|| dream_seed(&req.goal, ...).await);
    fs::write(run_dir.join("v01.rs"), seed).await?;

    let mut last_code = seed;
    let mut last_feedback: Option<PerceptionOutput> = None;
    let mut verdict = Verdict::Wrong;
    let mut n = 1u32;

    ui.emit(Phase::RunStarted, json!({})).await;

    while n <= req.max_iterations && verdict != Verdict::GoalAchieved {
        ui.emit(Phase::DecisionStarted, json!({"iteration": n})).await;
        let decision = decision::step(&req, n, &last_code, &last_feedback, &memory_hits, &mcp, &run_dir).await?;
        ui.emit(Phase::DecisionCompleted, json!({"reasoning": decision.reasoning, "tool_calls": decision.tool_calls})).await;

        // Apply tool calls in order. run_to_ppm + ppm_to_png must be last two.
        let ppm_path = apply_tool_calls(&decision.tool_calls, &mcp, &run_dir, n, &mut ui).await?;
        let png_path = run_dir.join(format!("v{:02}.png", n));

        ui.emit(Phase::RenderStarted, json!({"ppm": ppm_path})).await;
        mcp.call("ppm_to_png", json!({ "ppm_path": ppm_path, "png_path": png_path })).await?;
        ui.emit(Phase::RenderCompleted, json!({"png": png_path})).await;

        // Perception
        ui.emit(Phase::PerceptionStarted, json!({"iteration": n})).await;
        let perception = perception::step(&req, n, &png_path, &last_feedback).await?;
        ui.emit(Phase::PerceptionCompleted, json!({"verdict": perception.verdict, "critiques": perception.critiques})).await;

        // Persist feedback
        fs::write(run_dir.join(format!("v{:02}.feedback.json", n)), serde_json::to_vec_pretty(&perception)?).await?;

        // Memory update (heuristic)
        let new_mem = memory::maybe_record(&decision, &perception, &req.goal, &run_id);
        if let Some(item) = new_mem { memory::append(&item); }
        ui.emit(Phase::MemoryUpdated, json!({"added": new_mem})).await;

        // Save canonical versions: v01.rs, v02.rs, ...
        fs::write(run_dir.join(format!("v{:02}.rs", n)), &last_code_after_edits).await?;
        // (the apply_tool_calls already wrote to v{n}.rs; here we ensure canonical naming)

        verdict = perception.verdict.clone();
        last_feedback = Some(perception);

        // HIL pause
        if req.mode == Mode::Hil && verdict != Verdict::GoalAchieved {
            ui.emit(Phase::HILAwaitingApproval, json!({"iteration": n})).await;
            match ui.wait_for_hil_decision().await? {
                HilDecision::Approve => {},
                HilDecision::Edit(new_feedback) => { last_feedback = Some(new_feedback); },
                HilDecision::Stop => break,
            }
        }

        n = n + 1;
        last_code = fs::read_to_string(run_dir.join(format!("v{:02}.rs", n - 1))).await?;
    }

    ui.emit(Phase::RunDone, json!({"iterations": n - 1, "verdict": verdict})).await;
    Ok(RunSummary { run_id, iterations: n - 1, final_verdict: verdict })
}
```

### 7.2 Apply tool calls (the `apply_tool_calls` helper)

Walks the decision's `tool_calls` in order. For each:

- If `create_file` or `modify_file` → forward to MCP child, capture the
  resulting file content, write it to `state/runs/<run_id>/v{N}.rs`
  (overwriting each time). The decision agent's `path` argument may be
  any non-escaping path; the runner silently rewrites it to `v{N}.rs`
  before rendering so naming stays uniform. (We expose the rewrite in the
  decision prompt so the agent knows it doesn't have to worry about
  paths.)
- If `run_to_ppm` → forward to MCP child. Captures `ppm_path`.
- If `ppm_to_png` → skip; the runner invokes this itself right after
  `run_to_ppm` (single source of truth).
- On tool error → break out, send a corrective decision call (§4.4) with
  the error text. At most one corrective call per iteration.

### 7.3 Versioning & nomenclature (uniform across run)

Every artifact uses `v{:02}` (zero-padded to 2 digits, so `v01`..`v25`;
runs beyond 99 are capped at `v99`):
- `state/runs/<run_id>/v01.rs` … `v{N}.rs` — Rust source per iteration
- `state/runs/<run_id>/v01.ppm` … — PPM renders
- `state/runs/<run_id>/v01.png` … — PNG renders (the UI's chain)
- `state/runs/<run_id>/v01.feedback.json` — perception output
- `state/runs/<run_id>/v01.diff.patch` — unified diff from `v{N-1}.rs` to
  `v{N}.rs` (so the UI can show "what the agent changed"). Computed by the
  runner via a tiny in-process `diff` (we can pull `similar` crate for
  this, or shell out to `diff -u` for v1).

The `bin_name` passed to `run_to_ppm` is **always** `shape_composer_v{N}`
(e.g. `shape_composer_v03`). The runner materializes this bin into
`<workspace>/src/bin/shape_composer_v03.rs` (a copy of
`state/runs/<run_id>/v03.rs`), runs it, then deletes the bin file in a
`defer`/`Drop`. This keeps the workspace clean and ensures the agent can't
accidentally render a stale bin.

### 7.4 Render invocation details

```rust
async fn render_via_mcp(mcp: &McpChild, bin_name: &str, run_dir: &Path, n: u32) -> Result<PathBuf> {
    // 1. Materialize the bin into the workspace.
    let workspace_bin = workspace_root().join(format!("src/bin/{}.rs", bin_name));
    let src = fs::read(run_dir.join(format!("v{:02}.rs", n))).await?;
    let _guard = TempBinGuard::new(&workspace_bin, &src); // writes on new, deletes on drop

    // 2. Invoke run_to_ppm via MCP.
    let res = mcp.call("run_to_ppm", json!({ "bin_name": bin_name })).await?;
    let printed_ppm = res["ppm_path"].as_str().ok_or("no ppm_path")?.to_string();

    // 3. Move the PPM into run_dir with canonical name.
    let dst = run_dir.join(format!("v{:02}.ppm", n));
    fs::rename(&printed_ppm, &dst).await.ok().or_else(|_| fs::copy(&printed_ppm, &dst).await.ok());
    Ok(dst)
}
```

`TempBinGuard` is the Rust idiom for "create this file, do work, delete it
even on panic". The `Drop` impl calls `fs::remove_file` and ignores errors
(best-effort cleanup).

### 7.5 Termination

- `verdict == GoalAchieved` → runner exits the loop, emits `Phase::RunDone`
  with `success: true`.
- `n > MAX_ITER` (default 25, configurable, cap 60) → emits `Phase::RunDone`
  with `success: false`, `reason: "max_iterations"`.
- HIL: user clicks "Stop" → `HilDecision::Stop` → same as max_iterations.
- Unrecoverable error (gateway down, MCP child died, panic) → emits
  `Phase::Error` with `{reason, iteration: n}` and exits with code 1.

### 7.6 Run summary

On completion the runner writes `state/runs/<run_id>/summary.json`:

```jsonc
{
  "run_id": "run-abc12345",
  "goal": "...",
  "mode": "auto",
  "iterations": 7,
  "final_verdict": "goal_achieved",
  "final_png": "state/runs/run-abc12345/v07.png",
  "duration_s": 184.3,
  "memory_items_written": 3
}
```

The `live_server` reads this file for its "past runs" listing in the UI.

---

## 8. Frontend Integration (extends `live_server`)

### 8.1 Where it lives

We extend the existing Vite SPA in `live_server/web/`. There is no new
server process — `live_server` already runs Axum on `:3030` and serves the
SPA from `static/`. We add:

- A new top-level view `<div id="view-agent">` toggled by a nav link.
- New HTTP routes on `live_server` for agent ingest + SSE fanout.
- New TS module `agent.ts` for the SSE client + iteration gallery.

### 8.2 New `live_server` routes

```
POST  /agent/run                  → start a new run (body: RunRequest). Returns {run_id}.
                                    The runner is a child process spawned by live_server
                                    (or — alternative — the runner is launched out-of-band
                                    by the user as `cargo run --bin shape_composer -- ...`
                                    and POSTs to /agent/ingest). For v1 we prefer the
                                    out-of-band model: the user runs the agent in their
                                    terminal and the UI just observes.

POST  /agent/ingest/<run_id>      → agent pushes IterationEvent JSON. live_server stores
                                    it in memory (per-run event bus) and fans out to all
                                    SSE subscribers for that run_id. Body = IterationEvent.

GET   /agent/events/<run_id>      → SSE stream. Emits every IterationEvent received on
                                    /agent/ingest/<run_id> plus synthetic "replay" events
                                    sent on connect (any events the server already has
                                    buffered for that run_id, in order).

GET   /agent/runs                 → lists state/runs/*/summary.json as JSON array.
GET   /agent/runs/<run_id>        → returns summary.json + an index of all v{N}.rs, v{N}.png,
                                    v{N}.feedback.json for that run.
GET   /agent/runs/<run_id>/file   → ?kind=png|rs|feedback&n=3  — streams raw file.
                                    PNG with content-type image/png; rs with text/plain;
                                    feedback with application/json.

POST  /agent/runs/<run_id>/hil    → body {decision: "approve"|"stop"|"edit", feedback?}.
                                    Forwarded to the runner's HIL wait via a oneshot
                                    channel stored in the per-run in-memory state.
```

#### 8.2.1 Why out-of-band agent, in-process ingest?

Spawning `cargo run` from inside `live_server` binds the agent's lifetime
to the web server, which is fragile (long render loops should survive
browser refreshes). The out-of-band model decouples them: the user runs
the agent in a terminal; the agent POSTs to `live_server`; the browser
subscribes via SSE. The runner's `ui_ingest.rs` is a thin `reqwest`
client that retries on transient failures with a 1s backoff.

> The `POST /agent/run` route is still implemented for the future case
> where we want a "Start" button in the UI; it shells out to
> `cargo run --release --bin shape_composer -- --goal ... --mode auto` and
> returns immediately with the `run_id`. v1 leaves this unused; v2 wires
> the Start button.

### 8.3 `live_server` internal additions

```rust
// live_server/src/agent_state.rs ★ NEW
pub struct AgentBus {
    runs: Mutex<HashMap<String, RunState>>,
}
struct RunState {
    events: Vec<IterationEvent>,         // buffer for replay on SSE connect
    subscribers: Vec<tokio::sync::broadcast::Sender<IterationEvent>>,
    hil_reply: Option<oneshot::Sender<HilDecision>>,
}
impl AgentBus {
    pub fn ingest(&self, run_id: &str, ev: IterationEvent);
    pub fn subscribe(&self, run_id: &str) -> broadcast::Receiver<IterationEvent>;
    pub fn snapshot(&self, run_id: &str) -> Vec<IterationEvent>;
    pub fn set_hil_channel(&self, run_id: &str, tx: oneshot::Sender<HilDecision>);
    pub fn resolve_hil(&self, run_id: &str, decision: HilDecision) -> bool;
}
```

Add `AgentBus` to the Axum `Router` state. Wire the five new routes. The
existing `/ws`, `/scenes`, `/resolutions`, `/health` routes are untouched.

### 8.4 SPA changes (`live_server/web/`)

- `index.html` — add a top-level nav: `<a data-view="render">Render</a>` and
  `<a data-view="agent">Agent</a>`. The existing canvas view becomes the
  "render" view; the agent view is new.
- `app.ts` — add a tiny view-switcher; on "Agent" click, lazy-import
  `agent.ts` and mount it to `#view-agent`.
- `agent.ts` ★ NEW — does:
  1. `GET /agent/runs` → list past runs in a sidebar.
  2. On run click → `GET /agent/runs/<run_id>` for the index → render the
     "iteration gallery" (see §8.5).
  3. For a live run → `new EventSource('/agent/events/<run_id>')` → append
     a card per `Phase::IterationDone`, swap the "live" indicator.
  4. HIL controls (approve / edit / stop) → `POST /agent/runs/<run_id>/hil`.
- `agent.css` ★ NEW — gallery styling; images are shown **as full PNG**,
  one after another (no per-pixel rendering — see §8.6).

### 8.5 UI: the iteration gallery

```
┌─ Agent ─────────────────────────────────────────────────────────────┐
│  Goal: "a human figure made of boxes and cylinders"     [Auto|HIL]   │
│  Run: run-abc12345  •  iteration 4 / 25  •  verdict: partial  ●live │
│                                                                      │
│  ┌─ v01 ─┐ ┌─ v02 ─┐ ┌─ v03 ─┐ ┌─ v04 ─┐ ← click any to expand      │
│  │  PNG  │ │  PNG  │ │  PNG  │ │  PNG  │                                │
│  └───────┘ └───────┘ └───────┘ └───────┘                                │
│                                                                      │
│  ▼ v04 selected                                                      │
│  ┌──────────────────────┐  ┌──────────────────────────────────────┐ │
│  │                      │  │ Perception feedback (v04):          │ │
│  │   <img v04.png>      │  │ verdict: partial                     │ │
│  │                      │  │ critiques:                           │ │
│  │                      │  │  • the torso and head not close      │ │
│  │                      │  │  • upper cube too wide — reduce 25%  │ │
│  │                      │  │ suggestion_rank: 70                  │ │
│  └──────────────────────┘  └──────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Code diff (v03 → v04)                                          │  │
│  │ - cylinder.scaling(1.0, 1.0, 1.0)                              │  │
│  │ + cylinder.scaling(0.75, 1.0, 1.0)  // narrower upper cube     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  [Approve] [Edit feedback & retry] [Stop]   (HIL mode only)         │
└──────────────────────────────────────────────────────────────────────┘
```

Each card does NOT render pixel-by-pixel — it just sets
`<img src="/agent/runs/<run_id>/file?kind=png&n=4">`. The browser handles
the PNG display. The chain is the horizontal strip; selecting one shows
the big preview + the perception feedback + the code diff.

### 8.6 Performance note (explicit)

> "they do not have to render pixel by pixel right now. just one png after
> the other and the feedback the perception agent gave."

Confirmed: the agent tab uses `<img>` tags with server URLs, NOT the
existing per-pixel WebSocket renderer. The existing Render tab + `/ws`
endpoint stays exactly as-is for live streaming. The Agent tab is a
**separate, longitudinal** stream of complete PNGs + JSON feedback.

### 8.7 Protocol additions (`live_server/src/protocol.rs` and
`web/src/protocol.ts`)

Mirror the existing `PixelWire`/`FrameStart` types with new `Agent*`
types. We reuse the `IterationEvent` and `Phase` structs from
`shape_composer::schemas` (the agent serializes them, the server
forwards them, the SPA deserializes them — one shared shape). To avoid a
circular workspace dep, `live_server` declares the same structs locally
(they're tiny) and we keep them byte-identical via a unit test
(§11.7).

---

## 9. Knowledge Base (consumed by the decision agent)

### 9.1 Purpose

The decision agent needs to know:
- The exact public API of `ray_tracing_challenge_rs` (constructors,
  signatures, the idiomatic `shape.data.material.X = ...` pattern).
- The composition recipes that work (e.g. how `human_hand.rs` builds a
  finger from 3 cylinders chained by `&base * &translation * &rotation`).
- The "gotchas" (e.g. `Cylinder::minimum/maximum/closed` are **public
  fields**, not builder methods; `Material`'s custom `PartialEq` ignores
  several fields).

Without these, the agent hallucinates `Cylinder::new(length, radius)`
which doesn't exist. The KB is the antidote.

### 9.2 File layout

```
shape_composer/kb/
├── README.md                # how to use the KB; reads first
├── primitives.md            # sphere, cube, cylinder, cone, plane, triangle
├── transforms.md            # translation, scaling, rotation_x/y/z, shearing, view_transform
├── camera_and_canvas.md     # Camera::new, set_transform + view_transform, render, canvas_to_ppm
├── materials_and_patterns.md # Material fields, the 4 patterns, how to attach
├── lights.md                # PointLight, multiple lights, shadowing
├── groups_and_csg.md        # Group::add_child(Box::new(s)), nesting, CSG ops
└── composition_recipes.md   # 5 short worked examples (arm, leg, torso, head, ground plane)
```

Each file is ≤2k words so the prompt selector (§9.3) can pick several
without exceeding the 6000-char budget.

### 9.3 KB selection (`kb.rs::select_excerpts`)

For each decision call, select:

1. **Always**: `primitives.md`, `camera_and_canvas.md`.
2. **Goal-keyword-driven**: tokenize the goal; for each KB file compute a
   keyword-overlap score (same tokenizer as memory §6.4). Pick top 3
   files by score.
3. **Recipes**: always include 1 recipe from `composition_recipes.md`,
   chosen by goal keyword overlap. (Recipes are short sections within the
   one file.)
4. **Memory-driven**: if a memory hit references a KB file
  (`value.kb_ref == "groups_and_csg.md"`), include it.

Cap the concatenated excerpt at 6000 chars, truncating the lowest-scored
file first.

### 9.4 KB maintenance

The KB is plain markdown — the user can edit it freely. Future work
(§12.4): a `cargo run --bin kb_refresh` tool that walks `src/` and
re-emits `primitives.md` / `camera_and_canvas.md` / `materials_and_patterns.md`
by introspecting the actual source. For v1 the KB is hand-curated once
(derived from the exploration report in §1 of this plan).

### 9.5 `kb/README.md` draft

```md
# Shape Composer Knowledge Base

This directory contains short markdown references that the **decision
agent** reads before producing Rust code. The agent's prompt automatically
includes a subset of these files; the subset is chosen by goal keywords and
by process-memory hits.

Files are kept deliberately short (≤2k words each) because the agent prompt
budget for KB excerpts is ~6000 chars.

## Adding to the KB
- New file → must be referenced by `kb.rs::select_excerpts`'s static list.
- New section in an existing file → just append; sections are picked by
  goal-keyword overlap.
- Recipes go in `composition_recipes.md` under a `## <title>` heading.

## Stability contract
The API names referenced here MUST match `ray_tracing_challenge_rs`'s public
surface. If you rename a method in the library, update the KB the same day.
```

### 9.6 `kb/primitives.md` excerpt (so you can see the format)

```md
# Primitive shapes

All shapes implement the `Shape` trait. They wrap a public `data: ShapeData`
field whose `transform`, `material`, `parent` you can read or mutate
directly. Every shape gets a unique id at construction.

## Sphere
`Sphere::new() -> Sphere` — unit sphere at origin, default white material.
`Sphere::glass_sphere() -> Sphere` — pre-tuned glass (transparency=1.0,
refractive_index=1.5).
Set transform: `sphere.set_transform(translation(0,1,0));`
Mutate material: `sphere.data.material.color = Color::new(1,0,0);`
  (or `sphere.material_mut().color = ...`).

## Cube
`Cube::new() -> Cube` — axis-aligned unit cube centered at origin.
Same builder pattern as Sphere.

## Cylinder
`Cylinder::new() -> Cylinder` — infinite by default. **Truncate** by
mutating PUBLIC fields, no builder methods:
  let mut c = Cylinder::new();
  c.minimum = 0.0;
  c.maximum = 1.0;       // along Y in object space
  c.closed = true;       // add end caps
Orient it: `c.set_transform(&rotation_x(FRAC_PI_2) * &scaling(r, len, r));`
Units: object-space Y becomes world-space along `len`.

## Cone
Same as Cylinder: `Cone::new()`, set `minimum`, `maximum`, `closed` as
PUBLIC fields.

## Plane
`Plane::new() -> Plane` — infinite plane at y=0.

## Triangle
`Triangle::new(p1, p2, p3) -> Triangle` — takes three `Tuple::point`s.
Smooth variant: `SmoothTriangle::new(p1,p2,p3, n1,n2,n3)`.

## Group (for hierarchical composition)
`Group::new() -> Group`. Add a child:
  group.add_child(Box::new(sphere));
Children's `.parent` is auto-set. Groups can nest:
  hand.add_child(Box::new(finger_group));

## CSG (boolean composition)
`CSG::new(CSGOperation::Union, Box::new(a), Box::new(b))`.
Operations: `Union`, `Intersection`, `Difference`. Children's `.parent`
is auto-set.
```

(See also `composition_recipes.md` in §10 for the recipe examples.)

---

## 10. "Beautiful prompts" — sample goals the agent ships with

These are the default goals offered in the UI's "Goal" dropdown (and the
Rosetta-stone examples the `goal_dreamer` prompt can produce from a one-line
intent). Each is paired with a one-line hint of what success looks like.

1. **"A human figure made of boxes and cylinders, standing, arms relaxed"**
   - Hint: torso = elongated cube, head = cube, upper/lower arms and legs =
     truncated cylinders, hands/feet as small cubes.

2. **"A running figure mid-stride, leaning forward, captured in motion
   with primitives only"**
   - Hint: rotate torso ~15° about Z; rear leg lifted via knee cylinder
     rotation_x; front arm swung back.

3. **"A chess pawn on a checkered board"**
   - Hint: pawn = sphere on a truncated cone on a short cylinder; board =
     plane with `CheckersPattern`.

4. **"A small terrestrial planet with one moon, lit by a distant sun"**
   - Hint: two spheres, a point light far to one side, a starfield via a
     huge dark sphere.

5. **"A glass paperweight on a wooden desk, soft shadow"**
   - Hint: `Sphere::glass_sphere()` + multiple point lights for soft
     shadow + plane with low specular.

6. **"A totem pole of three stacked primitive heads"**
   - Hint: cubes with cylinder noses, cone headdresses, stacked along Y.

7. **"A simple house: cube body, pyramid (4-cone or 4-triangle) roof,
   door, window"**
   - Hint: cube + 4 triangles for a hip roof + small cube + plane for
     the door + thin cube indentation for window.

8. **"Bouncing ball: a sphere above a checkered plane, motion-blur streak
   suggested by 3 stacked translucent spheres"**
   - Hint: 3 spheres with decreasing transparency along an arc.

9. **"A robot DJ at a booth: head, torso, two arm cylinders, two turntable
   cubes, a reflective floor"**
   - Hint: cubes + cylinders, `material.reflective = 0.4` on the floor.

10. **"An open book standing on a table"**
    - Hint: two planes hinged at the spine (rotation_y ±0.3), thin cube
      spine, plane table.

### 10.1 The "goal dreamer" prompt (`prompts/goal_dreamer.md`)

Used when the user gives a one-liner ("something dynamic with primitives")
and the agent expands it into a fully-formed goal. NOT called in the main
loop — invoked once at run start if `req.goal` is too vague (heuristic:
<5 words). Returns a single string. Kept as a separate `.md` so we can
tune prompt-dreaming independently.

---

## 11. Build & Verification Recipe

### 11.1 Environment

- Rust stable (matches workspace edition 2024; rust ≥ 1.85).
- The LLM gateway running on `http://localhost:8108` (agent-spec §3). Start
  with `cd gateway && uv run main.py` (or whatever the project's gateway
  launch command is — verify in `gateway/README.md`).
- `live_server` running on `:3030` (existing): `cd live_server && cargo
  run --release`.
- Env vars:
  - `LLM_GATEWAY_URL` (default `http://localhost:8108`)
  - `GEMINI_API_KEY` (only used by the perception fallback §5.5; gateway
    path doesn't need it here)
  - `LIVE_SERVER_URL` (default `http://localhost:3030`)
  - `RUST_LOG=shape_composer=info,live_server=info`

### 11.2 Build steps

```bash
# from repo root
cargo build --release                       # builds the whole workspace
cargo run --release --bin live_server &     # :3030
cargo run --release --bin shape_composer -- \
    --goal "a human figure made of boxes and cylinders" \
    --mode auto \
    --max-iterations 25
```

### 11.3 Unit tests (crate-level, fast)

- `schemas.rs` — round-trip `DecisionOutput` and `PerceptionOutput` through
  `serde_json`; assert field names match the schema in §4.3 / §5.4.
- `memory.rs` — `read()` returns by token-overlap; `append()` is idempotent
  on identical ids.
- `kb.rs` — `select_excerpts(goal, mem)` always includes `primitives.md`;
  budget is respected.
- `prompt.rs` — all prompt blocks appear in order; tokens missing from
  context still render their placeholder rather than panic.
- `mcp_runner.rs` — `safe_resolve` rejects absolute, `..`, and symlink
  escapes (write a temp symlink to verify).
- `sandbox.rs::TempBinGuard` — Drop is called even on `panic!`.

### 11.4 Integration test (slow, requires gateway)

A `#[ignore]`-by-default test that:
1. Spawns the MCP child.
2. Calls `create_file` to write a tiny known-good bin.
3. Calls `run_to_ppm` then `ppm_to_png`.
4. Asserts `v01.png` exists and is >0 bytes.
5. Calls `perception::step` with a stub prompt; asserts a JSON response.
6. Cleans up `state/runs/<run_id>/`.

Run with `cargo test --release -- --ignored`.

### 11.5 End-to-end smoke

```bash
cargo run --release --bin shape_composer -- --goal "a red sphere on a plane" --mode auto --max-iterations 5
# then open http://localhost:3030/  → click "Agent" → select the just-created run
# → see iterations v01..v05 with PNGs + perception feedback
```

### 11.6 Pre-implementation verification (do this BEFORE writing the agent)

1. Confirm the gateway supports image input by sending a hand-crafted
   `POST /v1/chat` with an `image_url` content part. If it errors, the
   perception agent will use the §5.5 direct-Gemini fallback. Either way
   we proceed; just toggle the flag.
2. Confirm a freshly-generated bin's "Saved to …" stdout line matches what
   `run_to_ppm` parses. Reuse `three_spheres_on_plane.rs` as the test bin.
3. Confirm that the `image` crate's `ImageBuffer::<Rgb<u8>, _>::from_raw`
   + `.save(path)` path used in `src/bin/ppm_to_png.rs` works from
   `shape_composer` (it will — same crate family). This is the in-process
   `ppm_to_png` we port into the MCP tool.

### 11.7 Cross-crate protocol consistency test

A small test in `live_server` that imports JSON fixtures written by
`shape_composer`'s unit tests and deserializes them into the
live_server-local `IterationEvent` struct. Catches drift early.

---

## 12. Phased delivery

Slice the work into four shippable phases. Each phase is independently
valuable; later phases do not invalidate earlier ones.

### Phase 1 — The agent runs headless, no UI

- `shape_composer` crate skeleton (§2.1, §2.2).
- `schemas.rs`, `gw_client.rs` (chat only, no image).
- Decision agent end-to-end (§4) with a **stubbed perception agent** that
  always returns `verdict: partial, critiques: ["no feedback - stub"]`.
- MCP server with `create_file`, `modify_file`, `run_to_ppm`, `ppm_to_png`.
- `state/runs/<run_id>/vN.rs`, `vN.ppm`, `vN.png` on disk.
- KB first pass (§9).
- Outcome: `cargo run --bin shape_composer -- --goal "red sphere on
  plane" --mode auto --max-iterations 3` renders v01.png, v02.png, v03.png
  on disk. No UI. Verify by `open state/runs/<run_id>/v03.png`.

### Phase 2 — Real perception + process memory

- Replace stubbed perception with the Gemini-vision call (§5).
- Implement process memory (§6).
- Wire the corrective-hop path (§4.4) for real tool errors.
- Outcome: the agent now actually responds to feedback. Verify with
  "human figure" goal across 5–10 iterations; check that
  `memory/memory.json` grows by 1–2 items per run.

### Phase 3 — UI integration

- Add `live_server` routes + `AgentBus` (§8.2, §8.3).
- `ui_ingest.rs` in `shape_composer` (HTTP POST client, with retries).
- SPA: `agent.ts`, `agent.css`, gallery view, HIL controls.
- Outcome: open `http://localhost:3030/`, click "Agent", see the run's
  gallery live. Approve/Stop buttons work in HIL mode.

### Phase 4 — Polish & autonomy

- Goal dreamer (`prompts/goal_dreamer.md`) for vague prompts.
- KB auto-refresh tool (§9.4).
- Cross-run memory reasoning: prior-run hacks surface as memory hits in
  new runs.
- Past-runs sidebar in the SPA.
- Optional: parallel perception+decision on different stages (out of scope
  for v1; the loop is sequential by design).

---

## 13. Risks, scope cuts, out-of-scope

### 13.1 Risks

| Risk | Mitigation |
|---|---|
| Decision agent writes code that fails to compile | `run_to_ppm` returns `stderr_tail`; corrective hop feeds it back. MAX 1 corrective hop/iter → if stubborn, mark iteration `failed` and reuse previous code. |
| Gemini's vision API rejects the PNG (size / format) | `ppm_to_png` writes standard PNG; cap canvas size in the decision prompt to e.g. 400×400 for iterations 1..5 and 800×600 for later ones. The decision KB tells the agent how to pick `Camera::new(w,h,fov)`. |
| Compile times add up across iterations | Use `cargo run --release` (incremental). A tiny `.rs` bin rebuilds in <2s after the first run. Document the expected wall-clock per iteration (~15–30s incl. render + perception). |
| The agent "twitches" — oscillating between two compositions | Track the last 3 critiques; if the same critique flips sign 3× in a row, force a `Wrong` verdict + a "explore a different approach" perception hint. Log this in process memory as a `Hack`. |
| Hallucinated APIs in v01 | Strict KB inclusion (§9.3 #1 always includes `primitives.md`) + response_schema forcing structured output (no free-form code in the structured response — the code lives entirely in tool args, which the MCP `modify_file` will reject if the resulting file won't compile). |
| Gateway down | `gw_client` retries with backoff (3 attempts, 2s/4s/8s). After that, the run terminates with `Phase::Error`. Documented in the summary.json. |
| `live_server` restart loses in-memory event bus | The bus snapshot for replay is best-effort; for full history we read `state/runs/<run_id>/` from disk on `GET /agent/runs/<run_id>` (so SSE replay degrades to "what's on disk"). |

### 13.2 Explicit scope cuts (v1 does NOT include)

- No growing-DAG planner. We run a single perceive-decide-act loop (the
  rationale is in §1.1).
- No FAISS / vector embeddings. Memory recall is keyword-overlap only (§6).
- No multi-agent parallel dispatch. The loop is strictly sequential.
- No MCP server in a separate language. It is a Rust binary in the same
  crate (§3.1).
- No `browser` / `researcher` / `web_search` tools. The agent has all
  knowledge it needs locally (§9).
- No streaming-render UI for the Agent tab — PNGs are shown whole (this
  is per the user's explicit instruction; the existing Render tab stays
  for live pixel streaming).

### 13.3 What CAN extend later without rework

- Swap keyword memory for `usearch`-backed vector memory — the read API
  (§6.4) is already the right shape.
- Promote the perception agent to a second LLM profile (e.g. a stronger
  vision model) per run — `gw_client` already accepts `provider`/`model`.
- Allow the user to seed the run with their own `.rs` (already supported
  via `seed_prompt` in `RunRequest`).
- Add a 5th MCP tool (e.g. `read_bin` to peek at a prior version's code)
  — only requires editing the four-tool table and the `mcp_server.rs`
  dispatch. The decision prompt already advertises the tool list, so the
  model will see it.

---

## 14. Sketch of the most-used prompt (decision.md)

```md
You are the **decision agent** in a Rust ray-tracing composition loop.
You translate visual feedback from a perception agent into a small set of
MCP tool calls that edit ONE Rust source file and then render it to PNG.

# Strict rules
1. You emit a single JSON object matching the supplied schema. No prose
   outside the JSON.
2. `tool_calls` is an ordered list. Apply them sequentially. The last
   two calls MUST be `run_to_ppm` and then `ppm_to_png` (in that order).
3. The file you create/edit must:
   - be self-contained (only `use ray_tracing_challenge_rs::*` and `std`),
   - end with a `println!("Saved to ...")` line giving the absolute or
     `media/images_ppm/<name>.ppm` path,
   - call `Camera::new(width, height, fov)`, `camera.set_transform(
     view_transform(&from, &to, &up))`, `world.lights = vec![...]`,
     `world.add_shape(...)`, `camera.render(&world)`,
     `canvas.canvas_to_ppm()`, `std::fs::write(...)`.
4. Prefer small targeted `modify_file` edits over full rewrites. Only
   rewrite when perception verdict is `wrong` or when iteration is 1.
5. Stick to primitives from the KNOWLEDGE BASE EXCERPTS section. If you
   are unsure of a constructor signature, look it up there — do NOT
   guess.

# What you'll see
- GOAL, MODE, ITERATION, MAX_ITERATIONS
- PREVIOUS CODE (the current v{N-1}.rs)
- LAST FEEDBACK (perception's verdict + critiques + measurements from
  v{N-1}; in HIL mode, possibly overridden by the user)
- MEMORY HITS (hacks and pointers accumulated across runs)
- KNOWLEDGE BASE EXCERPTS (white-listed API names and idiom examples)
- AVAILABLE MCP TOOLS (exact tool argument schemas)

# How to think
- `reasoning`: short chain-of-thought (≤600 chars). What's perception
  asking for? Which primitive + transform achieves it?
- `tool_calls`: concrete calls. For `modify_file`, give specific `find`
  strings (verbatim from PREVIOUS CODE) and `replace` strings.
- `self_critique`: what could go wrong when these edits compile/render?
  This will be considered for long-term memory.

# Output JSON schema
(decision agent schema verbatim from §4.3)
```

---

## 15. Glossary

- **Decision agent** — text-only LLM that turns feedback into MCP tool calls.
- **Perception agent** — vision LLM (Gemini) that turns PNG + goal into
  verdict + critiques. The only component allowed to read images.
- **Iteration** — one pass of decision → render → perception → memory.
- **Run** — a full attempt to reach a goal, comprising 1..MAX_ITER
  iterations under a single `run_id`.
- **MCP** — Model Context Protocol; here, stdio JSON-RPC to a child
  binary with 4 tools.
- **Process memory** — cross-run append-only JSON list of hacks and
  pointers; keyword-recalled; injected into both agents' prompts.
- **KB** — Knowledge Base, hand-curated markdown about the ray tracer's
  API; goal-keyword-selected excerpts injected into the decision prompt.
- **HIL** — Human-in-the-Loop mode; the loop pauses after perception,
  awaits user approve/edit/stop.

---
(end of plan)
