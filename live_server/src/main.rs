//! Progressive render WebSocket server.
//!
//! Protocol (JSON, tagged by `"type"`):
//! - Client → server: `{ "type": "Start", "scene"?: "...", "width"?: N, "height"?: N, "mode": "...", "batch_size"?: N }`
//! - Server → client: `FrameStart { width, height }`, `Pixels { pixels: [...] }`, `FrameDone`
//!
//! Colors are `u8` in 0..=255 (same clamp as `Canvas::scale_component`).

mod agent_state;
mod protocol;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, Query, State},
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use agent_state::{AgentBus, AgentEvent};
use async_stream::stream;
use crossbeam_channel::unbounded;
use futures_util::StreamExt;
use protocol::{ClientMessage, PixelWire, RenderModeWire, ServerMessage};
use ray_tracing_challenge_rs::camera::{PixelUpdate, RenderMode};
use ray_tracing_challenge_rs::scenes;
use std::{convert::Infallible, net::SocketAddr, path::PathBuf};
use tower_http::{cors::CorsLayer, services::ServeDir};

/// Baked-in dev path (lives next to live_server/Cargo.toml). Overridable via env for Docker.
const DEFAULT_STATIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
const DEFAULT_SCENE: &str = "group_hexagon";

fn scale_component(c: f64) -> u8 {
    let scaled = (c * 255.0).round();
    scaled.clamp(0.0, 255.0) as u8
}

fn to_wire(p: PixelUpdate) -> PixelWire {
    PixelWire {
        x: p.x,
        y: p.y,
        r: scale_component(p.color.red),
        g: scale_component(p.color.green),
        b: scale_component(p.color.blue),
    }
}

fn default_batch_size(pixels: usize) -> usize {
    if pixels >= 1_000_000 {
        512
    } else {
        128
    }
}

#[tokio::main]
async fn main() {
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| DEFAULT_STATIC_DIR.into());

    let agent_bus = AgentBus::default();
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/scenes", get(scenes_handler))
        .route("/resolutions", get(resolutions_handler))
        .route("/agent/run", post(agent_start))
        .route("/agent/ingest/{run_id}", post(agent_ingest))
        .route("/agent/events/{run_id}", get(agent_events))
        .route("/agent/runs", get(agent_runs))
        .route("/agent/runs/{run_id}", get(agent_run))
        .route("/agent/runs/{run_id}/file", get(agent_file))
        .route("/agent/runs/{run_id}/hil", post(agent_hil))
        .fallback_service(ServeDir::new(&static_dir))
        .with_state(agent_bus)
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3030);
    let host = std::env::var("HOST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
    let addr = SocketAddr::new(host, port);

    println!("live_server listening on http://{addr}");
    println!("  static:  {static_dir}");
    println!("  ws:      ws://{addr}/ws  (wss:// behind a TLS-terminating proxy)");
    println!("  scenes:  {}", scenes::ids().join(", "));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}

fn agent_state_root() -> PathBuf {
    std::env::var_os("SHAPE_COMPOSER_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state"))
}

async fn agent_ingest(
    State(bus): State<AgentBus>,
    Path(run_id): Path<String>,
    Json(mut event): Json<AgentEvent>,
) -> impl IntoResponse {
    event.run_id = run_id;
    bus.ingest(event).await;
    Json(serde_json::json!({"ok": true}))
}

#[derive(serde::Deserialize)]
struct AgentStartRequest {
    goal: String,
    #[serde(default = "agent_auto_mode")]
    mode: String,
    #[serde(default = "agent_iterations")]
    max_iterations: u32,
}
fn agent_auto_mode() -> String { "auto".into() }
fn agent_iterations() -> u32 { 25 }
async fn agent_start(Json(request): Json<AgentStartRequest>) -> impl IntoResponse {
    if request.goal.trim().is_empty() || !matches!(request.mode.as_str(), "auto" | "hil") {
        return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"goal and valid mode are required"}))).into_response();
    }
    let run_id = format!("run-{}", &uuid_like());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let max_iterations = request.max_iterations.clamp(1, 60).to_string();
    let spawned = tokio::process::Command::new("cargo")
        .args(["run", "--release", "-p", "shape_composer", "--", "--goal", &request.goal, "--mode", &request.mode,
               "--max-iterations", &max_iterations])
        .current_dir(root)
        .env("SHAPE_COMPOSER_RUN_ID", &run_id)
        .env("LIVE_SERVER_URL", "http://127.0.0.1:3030")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => (axum::http::StatusCode::ACCEPTED, Json(serde_json::json!({"run_id":run_id}))).into_response(),
        Err(err) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":err.to_string()}))).into_response(),
    }
}
fn uuid_like() -> String {
    format!("{:08x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos())
}

async fn agent_events(
    State(bus): State<AgentBus>,
    Path(run_id): Path<String>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let (snapshot, mut receiver) = bus.snapshot_and_subscribe(&run_id).await;
    let events = stream! {
        for event in snapshot {
            yield Ok(Event::default().data(serde_json::to_string(&event).unwrap_or_default()));
        }
        loop {
            match receiver.recv().await {
                Ok(event) => yield Ok(Event::default().data(serde_json::to_string(&event).unwrap_or_default())),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Sse::new(events).keep_alive(KeepAlive::default())
}

async fn agent_runs() -> Json<serde_json::Value> {
    let root = agent_state_root().join("runs");
    let mut runs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            if let Ok(data) = std::fs::read(entry.path().join("summary.json")) {
                if let Ok(summary) = serde_json::from_slice::<serde_json::Value>(&data) { runs.push(summary); }
            }
        }
    }
    Json(serde_json::json!({"runs": runs}))
}

async fn agent_run(Path(run_id): Path<String>) -> impl IntoResponse {
    if run_id.contains('/') || run_id.contains("..") { return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid run id"}))).into_response(); }
    let dir = agent_state_root().join("runs").join(run_id);
    let summary = std::fs::read(dir.join("summary.json")).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let files: Vec<String> = std::fs::read_dir(&dir).ok().into_iter().flatten().flatten()
        .filter_map(|entry| entry.file_name().into_string().ok()).collect();
    (axum::http::StatusCode::OK, Json(serde_json::json!({"summary":summary,"files":files}))).into_response()
}

#[derive(serde::Deserialize)]
struct AgentFileQuery { kind: String, n: u32 }
async fn agent_file(Path(run_id): Path<String>, Query(q): Query<AgentFileQuery>) -> impl IntoResponse {
    if run_id.contains('/') || run_id.contains("..") { return axum::http::StatusCode::BAD_REQUEST.into_response(); }
    let ext = match q.kind.as_str() { "png" => "png", "rs" => "rs", "feedback" => "feedback.json", "diff" => "diff.patch", _ => return axum::http::StatusCode::BAD_REQUEST.into_response() };
    let file = agent_state_root().join("runs").join(run_id).join(format!("v{:02}.{}", q.n, ext));
    match tokio::fs::read(file).await {
        Ok(data) => ([(axum::http::header::CONTENT_TYPE, match q.kind.as_str() { "png" => "image/png", "feedback" => "application/json", _ => "text/plain; charset=utf-8" })], data).into_response(),
        Err(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn agent_hil(Path(_run_id): Path<String>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    // HIL replies are persisted as events; an out-of-band runner polls this endpoint in a later phase.
    Json(serde_json::json!({"ok": true, "received": body}))
}

async fn scenes_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "scenes": scenes::ids() }))
}

async fn resolutions_handler() -> Json<serde_json::Value> {
    let resolutions: Vec<serde_json::Value> = scenes::resolutions()
        .iter()
        .map(|(w, h, label)| {
            serde_json::json!({
                "label": label,
                "width": w,
                "height": h,
            })
        })
        .collect();
    Json(serde_json::json!({ "resolutions": resolutions }))
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let start = match wait_for_start(&mut socket).await {
        Some(ClientMessage::Start {
            mode,
            batch_size,
            scene,
            width,
            height,
        }) => (mode, batch_size, scene, width, height),
        None => return,
    };
    let (mode_wire, batch_size_opt, scene_opt, width_opt, height_opt) = start;

    let scene_id = scene_opt
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SCENE.to_string());

    let (w, h) = match (width_opt, height_opt) {
        (Some(w), Some(h)) => match scenes::parse_resolution(w, h) {
            Some(size) => size,
            None => {
                eprintln!("rejected resolution {w}x{h} (not in allowlist)");
                return;
            }
        },
        (None, None) => scenes::live_size(&scene_id),
        _ => {
            eprintln!("width and height must both be set or both omitted");
            return;
        }
    };

    let batch_size = batch_size_opt
        .unwrap_or_else(|| default_batch_size(w * h))
        .max(1);

    let mode = match mode_wire {
        RenderModeWire::Sequential => RenderMode::Sequential,
        RenderModeWire::Parallel => RenderMode::Parallel,
    };

    let Some((camera, world)) = scenes::build(&scene_id, w, h) else {
        eprintln!("unknown scene id: {scene_id}");
        return;
    };
    eprintln!("rendering scene={scene_id} {w}x{h} mode={mode:?} batch={batch_size}");

    let width = camera.hsize;
    let height = camera.vsize;
    let total = width * height;

    let frame_start = ServerMessage::FrameStart { width, height };
    if send_json(&mut socket, &frame_start).await.is_err() {
        return;
    }

    let (pixel_tx, pixel_rx) = unbounded::<PixelUpdate>();
    let (batch_tx, mut batch_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    std::thread::spawn(move || {
        camera.render_progressive(&world, mode, &pixel_tx);
    });

    std::thread::spawn(move || {
        let mut batch = Vec::with_capacity(batch_size);
        let mut sent = 0usize;
        while let Ok(pixel) = pixel_rx.recv() {
            batch.push(to_wire(pixel));
            if batch.len() >= batch_size {
                sent += batch.len();
                let msg = ServerMessage::Pixels {
                    pixels: std::mem::take(&mut batch),
                };
                if batch_tx.send(msg).is_err() {
                    return;
                }
                batch = Vec::with_capacity(batch_size);
            }
        }
        if !batch.is_empty() {
            sent += batch.len();
            let _ = batch_tx.send(ServerMessage::Pixels { pixels: batch });
        }
        let _ = batch_tx.send(ServerMessage::FrameDone);
        eprintln!("render complete: {sent}/{total} pixels");
    });

    while let Some(msg) = batch_rx.recv().await {
        if send_json(&mut socket, &msg).await.is_err() {
            break;
        }
    }
}

async fn wait_for_start(socket: &mut WebSocket) -> Option<ClientMessage> {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(start @ ClientMessage::Start { .. }) => return Some(start),
                Err(e) => {
                    eprintln!("invalid client message: {e}");
                }
            },
            Message::Close(_) => return None,
            _ => {}
        }
    }
    None
}

async fn send_json(socket: &mut WebSocket, msg: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|_| ())?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}
