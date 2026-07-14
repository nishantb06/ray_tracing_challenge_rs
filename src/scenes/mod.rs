//! Curated renderable scenes shared by CLI bins and live_server.

pub mod cover_scene;
pub mod football;
pub mod group_hexagon;
pub mod reflective_floor;
pub mod single_glass_sphere;

use crate::camera::Camera;
use crate::world::World;

pub const IDS: &[&str] = &[
    "group_hexagon",
    "cover_scene",
    "football",
    "single_glass_sphere",
    "reflective_floor",
];

/// Allowed live-render resolutions: `(width, height, label)`.
pub const RESOLUTIONS: &[(usize, usize, &str)] = &[
    (400, 400, "400 × 400"),
    (300, 400, "300 × 400"),
    (1200, 800, "1200 × 800"),
    (1920, 1280, "1920 × 1280"),
    (3840, 2160, "3840 × 2160 (4K)"),
];

pub fn ids() -> &'static [&'static str] {
    IDS
}

pub fn resolutions() -> &'static [(usize, usize, &'static str)] {
    RESOLUTIONS
}

/// Accept only allowlisted sizes. Returns `None` for anything else.
pub fn parse_resolution(width: usize, height: usize) -> Option<(usize, usize)> {
    RESOLUTIONS
        .iter()
        .find(|(w, h, _)| *w == width && *h == height)
        .map(|(w, h, _)| (*w, *h))
}

/// Fallback when the client omits width/height.
pub fn live_size(id: &str) -> (usize, usize) {
    match id {
        "group_hexagon" | "cover_scene" => (400, 400),
        _ => (400, 400), // use default preset when height previously differed
    }
}

pub fn build(id: &str, width: usize, height: usize) -> Option<(Camera, World)> {
    match id {
        "group_hexagon" => Some(group_hexagon::build(width, height)),
        "cover_scene" => Some(cover_scene::build(width, height)),
        "football" => Some(football::build(width, height)),
        "single_glass_sphere" => Some(single_glass_sphere::build(width, height)),
        "reflective_floor" => Some(reflective_floor::build(width, height)),
        _ => None,
    }
}
