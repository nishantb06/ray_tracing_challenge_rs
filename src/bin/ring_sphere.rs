use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{Pattern, RingPattern};
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, scaling, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Unit sphere with concentric green rings.
    let mut ring_pattern = RingPattern::new(
        Color::new(0.1, 0.9, 0.1), // light green
        Color::new(0.0, 0.3, 0.0), // dark green
    );
    // Turn the pattern slightly toward -z and scale down so more rings are visible.
    let transform = &scaling(0.1, 0.1, 0.1) * &rotation_x(-std::f64::consts::FRAC_PI_3);
    ring_pattern.set_transform(transform);

    let mut sphere = Sphere::new();
    sphere.data.material = Material::new();
    sphere.data.material.color = Color::new(0.1, 0.9, 0.1);
    sphere.data.material.diffuse = 0.7;
    sphere.data.material.specular = 0.3;
    sphere.data.material.pattern = Some(Box::new(ring_pattern));

    let mut world = World::new();
    world.add_shape(sphere);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(800, 400, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 0.0, -5.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/ring_sphere.ppm", ppm)
        .expect("Failed to write ring_sphere.ppm");
    println!("Saved to media/images_ppm/ring_sphere.ppm");
}

