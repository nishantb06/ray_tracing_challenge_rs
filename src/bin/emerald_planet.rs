use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // --- Ground planet surface ---
    let mut ground = Sphere::new();
    ground.set_transform(scaling(50.0, 0.01, 50.0));
    ground.data.material = Material::new();
    ground.data.material.color = Color::new(0.05, 0.05, 0.08);
    ground.data.material.specular = 0.0;

    // --- Giant emerald sphere ---
    let mut emerald = Sphere::new();
    emerald.set_transform(translation(0.0, 1.5, 0.0));
    emerald.data.material = Material::new();
    emerald.data.material.color = Color::new(0.0, 0.9, 0.6);
    emerald.data.material.diffuse = 0.7;
    emerald.data.material.specular = 0.9;

    // --- Small orbit sphere 1 ---
    let mut orb1 = Sphere::new();
    orb1.set_transform(&translation(2.5, 0.5, -1.0) * &scaling(0.5, 0.5, 0.5));
    orb1.data.material = Material::new();
    orb1.data.material.color = Color::new(0.9, 0.3, 0.6);
    orb1.data.material.diffuse = 0.7;
    orb1.data.material.specular = 0.6;

    // --- Small orbit sphere 2 ---
    let mut orb2 = Sphere::new();
    orb2.set_transform(&translation(-2.0, 0.4, 1.0) * &scaling(0.4, 0.4, 0.4));
    orb2.data.material = Material::new();
    orb2.data.material.color = Color::new(0.2, 0.6, 1.0);
    orb2.data.material.diffuse = 0.7;
    orb2.data.material.specular = 0.8;

    // --- Tiny bright sphere ---
    let mut orb3 = Sphere::new();
    orb3.set_transform(&translation(0.8, 0.25, 2.2) * &scaling(0.25, 0.25, 0.25));
    orb3.data.material = Material::new();
    orb3.data.material.color = Color::new(1.0, 0.8, 0.3);
    orb3.data.material.diffuse = 0.7;
    orb3.data.material.specular = 1.0;

    // --- World ---
    let mut world = World::new();
    world.add_shape(ground);
    world.add_shape(emerald);
    world.add_shape(orb1);
    world.add_shape(orb2);
    world.add_shape(orb3);
    world.lights = vec![PointLight::new(
        Tuple::point(-15.0, 20.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // --- Camera ---
    let mut camera = Camera::new(1000, 600, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 4.0, -10.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();

    std::fs::write("media/images_ppm/emerald_planet.ppm", ppm)
        .expect("Failed to write emerald_planet.ppm");

    println!("Saved to media/images_ppm/emerald_planet.ppm");
}
