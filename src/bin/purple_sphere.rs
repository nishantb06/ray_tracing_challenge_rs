use ray_tracing_challenge_rs::canvas::{Canvas, Color};
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::lighting;
use ray_tracing_challenge_rs::ray::Ray;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::scaling;
use ray_tracing_challenge_rs::tuple::Tuple;

fn main() {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_pixels = 500.0;

    let pixel_size = wall_size / canvas_pixels;
    let half = wall_size / 2.0;

    let mut canvas = Canvas::new(canvas_pixels as usize, canvas_pixels as usize);

    let mut shape = Sphere::new();
    shape.data.material.color = Color::new(1.0, 0.2, 1.0);
    shape.set_transform(scaling(0.5, 1.0, 1.0));
    // // ellipse wide and flat
    // shape.set_transform(scaling(1.0, 0.5, 1.0));

    // let m = &scaling(0.5, 1.0, 1.0) * &rotation_z(std::f64::consts::FRAC_PI_4);
    // shape.set_transform(m);

    // shape.set_transform(shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0));

    // shape.set_transform(translation(1.0, 0.5, 0.0));
    let light = PointLight::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

    // let t = &translation(0.5, 0.0, 0.0) * &(&scaling(0.5, 1.0, 1.0) * &rotation_z(0.8));
    // shape.set_transform(t);

    for y in 0..canvas.height {
        let world_y = half - pixel_size * y as f64;
        for x in 0..canvas.width {
            let world_x = -half + pixel_size * x as f64;
            let wall_point = Tuple::point(world_x, world_y, wall_z);

            let direction = (&wall_point - &ray_origin).normalize();
            let r = Ray::new(ray_origin.clone(), direction);
            let xs = shape.intersect(&r);

            if let Some(hit) = xs.hit() {
                let point = r.position(hit.t);
                let normal = hit.object.normal_at(&point);
                let eye = -&r.direction;
                let color = lighting(hit.object.material(), &light, &point, &eye, &normal, false);
                canvas.write_pixel(x, y, color);
            }
        }
    }

    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/purple_sphere.ppm", ppm)
        .expect("Failed to write purple_sphere.ppm");
    println!("Saved purple sphere to media/images_ppm/purple_sphere.ppm");
}
