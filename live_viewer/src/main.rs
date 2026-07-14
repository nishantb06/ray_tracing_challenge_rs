//! egui viewer for progressive pixel streaming over WebSocket.
//!
//! Mirrors the JSON protocol defined by `live_server` (no raytracer dependency).

mod protocol;

use eframe::egui;
use futures_util::{SinkExt, StreamExt};
use protocol::{ClientMessage, RenderModeWire, ServerMessage};
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const WS_URL: &str = "ws://127.0.0.1:3030/ws";

struct SharedFrame {
    width: usize,
    height: usize,
    /// RGBA8, row-major.
    pixels: Vec<u8>,
    done: usize,
    total: usize,
    finished: bool,
    status: String,
    /// Bumps when pixels change so the UI can refresh the egui texture.
    generation: u64,
}

impl SharedFrame {
    fn pending(status: &str) -> Self {
        Self {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            done: 0,
            total: 0,
            finished: false,
            status: status.to_string(),
            generation: 0,
        }
    }
}

struct LiveViewerApp {
    frame: Arc<Mutex<SharedFrame>>,
    texture: Option<egui::TextureHandle>,
    last_generation: u64,
    mode: RenderModeWire,
    started: bool,
}

impl LiveViewerApp {
    fn new(cc: &eframe::CreationContext<'_>, mode: RenderModeWire) -> Self {
        let frame = Arc::new(Mutex::new(SharedFrame::pending("Connecting…")));
        let frame_clone = Arc::clone(&frame);
        let ctx = cc.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(run_client(frame_clone, ctx, mode));
        });

        Self {
            frame,
            texture: None,
            last_generation: 0,
            mode,
            started: true,
        }
    }
}

impl eframe::App for LiveViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let snapshot = {
            let g = self.frame.lock().unwrap();
            (
                g.width,
                g.height,
                g.pixels.clone(),
                g.done,
                g.total,
                g.finished,
                g.status.clone(),
                g.generation,
            )
        };
        let (width, height, pixels, done, total, finished, status, generation) = snapshot;

        if width > 0 && height > 0 && !pixels.is_empty() {
            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels);
            if self.texture.is_none() || generation != self.last_generation {
                self.texture =
                    Some(ctx.load_texture("live_render", color_image, Default::default()));
                self.last_generation = generation;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live ray tracer");
            ui.label(format!(
                "Mode: {:?}  |  {status}",
                self.mode
            ));
            if total > 0 {
                ui.label(format!("Progress: {done} / {total}"));
                let frac = done as f32 / total as f32;
                ui.add(egui::ProgressBar::new(frac).show_percentage());
            }
            if finished {
                ui.label("Frame complete.");
            }
            if !self.started {
                ui.label("Waiting to start…");
            }

            if let Some(tex) = &self.texture {
                let max = ui.available_size();
                let aspect = width as f32 / height.max(1) as f32;
                let mut size = max;
                if size.x / aspect <= size.y {
                    size.y = size.x / aspect;
                } else {
                    size.x = size.y * aspect;
                }
                ui.image((tex.id(), size));
            } else {
                ui.label("Waiting for FrameStart…");
            }
        });

        // Keep repainting while the stream is active.
        if !finished {
            ctx.request_repaint();
        }
    }
}

async fn run_client(frame: Arc<Mutex<SharedFrame>>, ctx: egui::Context, mode: RenderModeWire) {
    {
        let mut g = frame.lock().unwrap();
        g.status = format!("Connecting to {WS_URL}");
    }
    ctx.request_repaint();

    let (ws, _) = match connect_async(WS_URL).await {
        Ok(v) => v,
        Err(e) => {
            let mut g = frame.lock().unwrap();
            g.status = format!("Connection failed: {e}. Is live_server running?");
            ctx.request_repaint();
            return;
        }
    };

    let (mut write, mut read) = ws.split();

    let start = ClientMessage::Start {
        mode,
        batch_size: Some(128),
        scene: None,
        width: None,
        height: None,
    };
    let text = serde_json::to_string(&start).expect("serialize Start");
    if let Err(e) = write.send(Message::Text(text.into())).await {
        let mut g = frame.lock().unwrap();
        g.status = format!("Failed to send Start: {e}");
        ctx.request_repaint();
        return;
    }

    {
        let mut g = frame.lock().unwrap();
        g.status = "Rendering…".into();
    }

    while let Some(msg) = read.next().await {
        let Ok(msg) = msg else { break };
        let Message::Text(text) = msg else { continue };
        let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) else {
            continue;
        };

        match server_msg {
            ServerMessage::FrameStart { width, height } => {
                let mut g = frame.lock().unwrap();
                g.width = width;
                g.height = height;
                g.total = width * height;
                g.done = 0;
                g.finished = false;
                g.pixels = vec![0u8; width * height * 4];
                g.generation += 1;
                g.status = format!("Frame {width}×{height}");
            }
            ServerMessage::Pixels { pixels } => {
                let mut g = frame.lock().unwrap();
                for p in pixels {
                    if p.x < g.width && p.y < g.height {
                        let i = (p.y * g.width + p.x) * 4;
                        g.pixels[i] = p.r;
                        g.pixels[i + 1] = p.g;
                        g.pixels[i + 2] = p.b;
                        g.pixels[i + 3] = 255;
                        g.done += 1;
                    }
                }
                g.generation += 1;
            }
            ServerMessage::FrameDone => {
                let mut g = frame.lock().unwrap();
                g.finished = true;
                g.status = "Done".into();
                g.generation += 1;
                ctx.request_repaint();
                break;
            }
        }
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let mode = parse_mode(std::env::args().nth(1).as_deref());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 780.0])
            .with_title("Live ray tracer"),
        ..Default::default()
    };
    eframe::run_native(
        "Live ray tracer",
        options,
        Box::new(move |cc| Ok(Box::new(LiveViewerApp::new(cc, mode)))),
    )
}

fn parse_mode(arg: Option<&str>) -> RenderModeWire {
    match arg {
        Some("parallel") | Some("--parallel") => RenderModeWire::Parallel,
        _ => RenderModeWire::Sequential,
    }
}
