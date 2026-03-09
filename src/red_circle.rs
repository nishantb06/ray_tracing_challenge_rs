mod tuple;
mod utils;
mod matrix;
mod transformation;
mod canvas;
mod ray;
mod intersection;
mod sphere;

use canvas::{Canvas, Color};
use tuple::Tuple;
use sphere::Sphere;
use ray::Ray;

fn main() {
    let ray_origin = Tuple::point(0.0,0.0,-5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_pixels = 100.0;

    // to get the size of a single pixel in world space units
    let pixel_size = wall_size / canvas_pixels;

    // this variable describes the minimum and maximum x and y coordinates of the wall
    let half = wall_size/2.0;

    let mut canvas = Canvas::new(canvas_pixels as usize, canvas_pixels as usize);
    let red = Color::new(1.0, 0.0, 0.0);
    let shape = Sphere::new();
    println!("{:?}", ray_origin);

    // for each row of pixels in the canvas
    for y in 0..canvas.height {
        // compute the world y coordinate (top = +half, bottom = -half)
        let world_y = half - pixel_size * y as f64;
        // for each pixel in the row
        for x in 0..canvas.width {
            let world_x = half - pixel_size * x as f64;
            let position = Tuple::point(world_x,world_y,wall_z);

            // After (correct):
            let r = Ray::new(ray_origin.clone(), (&position - &ray_origin).normalize());
            let xs = shape.intersect(&r);

            if let Some(i) = xs.hit() {
                canvas.write_pixel(x, y, red.clone());
            }
        }
    }

    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/red_circle.ppm", ppm).expect("Failed to write red circle.ppm");
    println!("Saved red circle face to media/images_ppm/red_circle.ppm");
}
