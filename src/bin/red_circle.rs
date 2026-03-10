use ray_tracing_challenge_rs::canvas::{Canvas, Color};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::ray::Ray;

fn main() {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_pixels = 100.0;

    let pixel_size = wall_size / canvas_pixels;
    let half = wall_size / 2.0;

    let mut canvas = Canvas::new(canvas_pixels as usize, canvas_pixels as usize);
    let red = Color::new(1.0, 0.0, 0.0);
    let shape = Sphere::new();

    for y in 0..canvas.height {
        let world_y = half - pixel_size * y as f64;
        for x in 0..canvas.width {
            let world_x = -half + pixel_size * x as f64;
            let position = Tuple::point(world_x, world_y, wall_z);

            let r = Ray::new(ray_origin.clone(), (&position - &ray_origin).normalize());
            let xs = shape.intersect(&r);

            if let Some(_hit) = xs.hit() {
                canvas.write_pixel(x, y, red.clone());
            }
        }
    }

    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/red_circle.ppm", ppm).expect("Failed to write red_circle.ppm");
    println!("Saved red circle to media/images_ppm/red_circle.ppm");
}
