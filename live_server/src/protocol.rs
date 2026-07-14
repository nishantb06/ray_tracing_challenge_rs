use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    FrameStart { width: usize, height: usize },
    Pixels { pixels: Vec<PixelWire> },
    FrameDone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixelWire {
    pub x: usize,
    pub y: usize,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Start {
        mode: RenderModeWire,
        #[serde(default)]
        batch_size: Option<usize>,
        #[serde(default)]
        scene: Option<String>,
        #[serde(default)]
        width: Option<usize>,
        #[serde(default)]
        height: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderModeWire {
    Sequential,
    Parallel,
}
