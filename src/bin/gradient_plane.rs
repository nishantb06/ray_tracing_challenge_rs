use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{GradientPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::transformation::{scaling, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Gradient on the X axis, applied to an infinite plane in the XZ plane.
    // Colors: deep blue -> bright magenta
    let mut grad = GradientPattern::new(
        Color::new(0.0, 0.0, 0.4),
        Color::new(1.0, 0.0, 1.0),
    );

    // Make the gradient change slowly across the plane:
    // small scale in X so a large world distance maps into 0..1 in pattern space.
    grad.set_transform(scaling(0.1, 1.0, 1.0));

    let mut plane = Plane::new();
    plane.data.material = Material::new();
    plane.data.material.color = Color::new(1.0, 1.0, 1.0);
    plane.data.material.ambient = 0.2;
    plane.data.material.diffuse = 0.8;
    plane.data.material.specular = 0.0;
    plane.data.material.pattern = Some(Box::new(grad));

    let mut world = World::new();
    world.add_shape(plane);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 2.0, -3.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/gradient_plane.ppm", ppm)
        .expect("Failed to write gradient_plane.ppm");
    println!("Saved to media/images_ppm/gradient_plane.ppm");
}

