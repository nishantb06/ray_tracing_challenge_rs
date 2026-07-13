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
    // ---- Background ground (space plane) ----
    let mut space = Sphere::new();
    space.set_transform(scaling(100.0, 0.01, 100.0));
    space.data.material = Material::new();
    space.data.material.color = Color::new(0.02, 0.02, 0.05);
    space.data.material.specular = 0.0;

    // ---- Sun ----
    let mut sun = Sphere::new();
    sun.set_transform(translation(0.0, 3.0, 4.0));
    sun.data.material = Material::new();
    sun.data.material.color = Color::new(1.0, 0.8, 0.2);
    sun.data.material.diffuse = 0.9;
    sun.data.material.specular = 1.0;

    // ---- Mercury ----
    let mut mercury = Sphere::new();
    mercury.set_transform(&translation(-4.0, 0.3, 0.5) * &scaling(0.2, 0.2, 0.2));
    mercury.data.material = Material::new();
    mercury.data.material.color = Color::new(0.7, 0.7, 0.7);

    // ---- Venus ----
    let mut venus = Sphere::new();
    venus.set_transform(&translation(-3.0, 0.35, 1.2) * &scaling(0.3, 0.3, 0.3));
    venus.data.material = Material::new();
    venus.data.material.color = Color::new(1.0, 0.7, 0.3);

    // ---- Earth ----
    let mut earth = Sphere::new();
    earth.set_transform(&translation(-1.8, 0.4, 1.8) * &scaling(0.35, 0.35, 0.35));
    earth.data.material = Material::new();
    earth.data.material.color = Color::new(0.2, 0.4, 1.0);

    // ---- Mars ----
    let mut mars = Sphere::new();
    mars.set_transform(&translation(-0.8, 0.3, 2.5) * &scaling(0.28, 0.28, 0.28));
    mars.data.material = Material::new();
    mars.data.material.color = Color::new(0.9, 0.3, 0.2);

    // ---- Jupiter ----
    let mut jupiter = Sphere::new();
    jupiter.set_transform(&translation(0.8, 0.9, 2.0) * &scaling(0.9, 0.9, 0.9));
    jupiter.data.material = Material::new();
    jupiter.data.material.color = Color::new(0.9, 0.7, 0.5);

    // ---- Saturn ----
    let mut saturn = Sphere::new();
    saturn.set_transform(&translation(2.4, 0.75, 1.2) * &scaling(0.75, 0.75, 0.75));
    saturn.data.material = Material::new();
    saturn.data.material.color = Color::new(0.9, 0.8, 0.5);

    // ---- Uranus ----
    let mut uranus = Sphere::new();
    uranus.set_transform(&translation(3.6, 0.5, 0.4) * &scaling(0.45, 0.45, 0.45));
    uranus.data.material = Material::new();
    uranus.data.material.color = Color::new(0.5, 0.9, 0.9);

    // ---- Neptune ----
    let mut neptune = Sphere::new();
    neptune.set_transform(&translation(4.7, 0.45, -0.3) * &scaling(0.42, 0.42, 0.42));
    neptune.data.material = Material::new();
    neptune.data.material.color = Color::new(0.2, 0.3, 0.9);

    // ---- Pluto ----
    let mut pluto = Sphere::new();
    pluto.set_transform(&translation(5.6, 0.2, -1.0) * &scaling(0.18, 0.18, 0.18));
    pluto.data.material = Material::new();
    pluto.data.material.color = Color::new(0.8, 0.7, 0.6);

    // ---- World ----
    let mut world = World::new();
    world.add_shape(space);
    world.add_shape(sun);
    world.add_shape(mercury);
    world.add_shape(venus);
    world.add_shape(earth);
    world.add_shape(mars);
    world.add_shape(jupiter);
    world.add_shape(saturn);
    world.add_shape(uranus);
    world.add_shape(neptune);
    world.add_shape(pluto);
    world.lights = vec![PointLight::new(
        Tuple::point(0.0, 6.0, -8.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // ---- Camera ----
    let mut camera = Camera::new(1200, 600, FRAC_PI_3);

    camera.set_transform(view_transform(
        &Tuple::point(0.0, 4.5, -12.0),
        &Tuple::point(0.0, 1.0, 1.5),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();

    std::fs::write("media/images_ppm/solar_system.ppm", ppm)
        .expect("Failed to write solar_system.ppm");

    println!("Saved to media/images_ppm/solar_system.ppm");
}
