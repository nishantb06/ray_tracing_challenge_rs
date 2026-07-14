//! Progressive render WebSocket server.
//!
//! Protocol (JSON, tagged by `"type"`):
//! - Client → server: `{ "type": "Start", "scene"?: "...", "width"?: N, "height"?: N, "mode": "...", "batch_size"?: N }`
//! - Server → client: `FrameStart { width, height }`, `Pixels { pixels: [...] }`, `FrameDone`
//!
//! Colors are `u8` in 0..=255 (same clamp as `Canvas::scale_component`).

mod protocol;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use crossbeam_channel::unbounded;
use futures_util::StreamExt;
use protocol::{ClientMessage, PixelWire, RenderModeWire, ServerMessage};
use ray_tracing_challenge_rs::camera::{PixelUpdate, RenderMode};
use ray_tracing_challenge_rs::scenes;
use std::net::SocketAddr;
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

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/scenes", get(scenes_handler))
        .route("/resolutions", get(resolutions_handler))
        .fallback_service(ServeDir::new(&static_dir))
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
