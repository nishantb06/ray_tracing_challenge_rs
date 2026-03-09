mod tuple;
mod utils;
mod matrix;
mod canvas;
mod transformation;

use canvas::{Canvas, Color};
use tuple::Tuple;
use transformation::rotation_y;
use std::f64::consts::PI;

fn main() {
    let size: usize = 500;
    let mut c = Canvas::new(size, size);
    let white = Color::new(1.0, 1.0, 1.0);

    let radius = size as f64 * 3.0 / 8.0;
    let center = size as f64 / 2.0;
    let twelve = Tuple::point(0.0, 0.0, 1.0);

    for hour in 0..12 {
        let r = rotation_y(hour as f64 * PI / 6.0);
        let pos = &r * &twelve;

        let px = (center + pos.x * radius).round() as usize;
        let py = (center - pos.z * radius).round() as usize;

        for dy in 0..3_usize {
            for dx in 0..3_usize {
                let x = px.saturating_add(dx).saturating_sub(1).min(size - 1);
                let y = py.saturating_add(dy).saturating_sub(1).min(size - 1);
                c.write_pixel(x, y, white.clone());
            }
        }
    }

    let ppm = c.canvas_to_ppm();
    std::fs::write("media/images/clock.ppm", ppm).expect("Failed to write clock.ppm");
    println!("Saved clock face to media/images/clock.ppm");
}
