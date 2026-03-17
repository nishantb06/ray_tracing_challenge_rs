use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::transformation::{scaling, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Infinite plane in XZ with classic black/white checkers.
    let mut checkers = CheckersPattern::new(
        Color::new(0.9, 0.9, 0.9),
        Color::new(0.1, 0.1, 0.1),
    );
    // Coarse pattern so checks are easy to see and not too aliased.
    checkers.set_transform(scaling(1.5, 1.0, 1.5));

    let mut plane = Plane::new();
    plane.data.material = Material::new();
    plane.data.material.color = Color::new(1.0, 1.0, 1.0);
    plane.data.material.ambient = 0.2;
    plane.data.material.diffuse = 0.8;
    plane.data.material.specular = 0.0;
    plane.data.material.pattern = Some(Box::new(checkers));

    let mut world = World::new();
    world.add_shape(plane);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 15.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 8.0, -12.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/checker_plane.ppm", ppm)
        .expect("Failed to write checker_plane.ppm");
    println!("Saved to media/images_ppm/checker_plane.ppm");
}

