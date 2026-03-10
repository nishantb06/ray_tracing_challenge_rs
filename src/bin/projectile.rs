use ray_tracing_challenge_rs::canvas::{Canvas, Color};
use ray_tracing_challenge_rs::tuple::Tuple;
use std::fs;

struct Projectile {
    position: Tuple,
    velocity: Tuple,
}

struct Environment {
    gravity: Tuple,
    wind: Tuple,
}

fn tick(env: &Environment, proj: &Projectile) -> Projectile {
    let position = &proj.position + &proj.velocity;
    let gravity_and_wind = &env.gravity + &env.wind;
    let velocity = &proj.velocity + &gravity_and_wind;
    Projectile { position, velocity }
}

fn main() {
    let velocity = &Tuple::vector(1.0, 1.0, 0.0).normalize() * 11.25;
    let projectile = Projectile {
        position: Tuple::point(0.0, 1.0, 0.0),
        velocity,
    };

    let environment = Environment {
        gravity: Tuple::vector(0.0, -0.1, 0.0),
        wind: Tuple::vector(-0.01, 0.0, 0.0),
    };

    let mut positions = vec![];
    let mut proj = projectile;
    while proj.position.y > 0.0 {
        positions.push((proj.position.x, proj.position.y));
        proj = tick(&environment, &proj);
    }

    let min_x = positions.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = positions.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = positions.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = positions.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

    let padding = 10.0;
    let world_width = (max_x - min_x + 2.0 * padding).max(1.0);
    let world_height = (max_y - min_y + 2.0 * padding).max(1.0);

    let canvas_width = 900;
    let canvas_height = 550;

    let mut canvas = Canvas::new(canvas_width, canvas_height);
    let red = Color::new(1.0, 0.0, 0.0);

    for (wx, wy) in positions {
        let cx = ((wx - min_x + padding) / world_width * (canvas_width as f64)).round() as usize;
        let cy = ((max_y - wy + padding) / world_height * (canvas_height as f64)).round() as usize;
        if cx < canvas_width && cy < canvas_height {
            canvas.write_pixel(cx, cy, red.clone());
        }
    }

    let ppm = canvas.canvas_to_ppm();
    fs::write("trajectory.ppm", ppm).expect("Failed to write PPM file");
    println!("Saved trajectory.ppm");
}
