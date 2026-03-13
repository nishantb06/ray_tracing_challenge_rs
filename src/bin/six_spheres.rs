use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4};

fn main() {
    // Floor
    let mut floor = Sphere::new();
    floor.set_transform(scaling(10.0, 0.01, 10.0));
    floor.material = Material::new();
    floor.material.color = Color::new(1.0, 0.9, 0.9);
    floor.material.specular = 0.0;

    // Left wall
    let mut left_wall = Sphere::new();
    left_wall.set_transform(
        &(&(&translation(0.0, 0.0, 5.0) * &rotation_y(-FRAC_PI_4)) * &rotation_x(FRAC_PI_2))
            * &scaling(10.0, 0.01, 10.0),
    );
    left_wall.material = floor.material.clone();

    // Right wall
    let mut right_wall = Sphere::new();
    right_wall.set_transform(
        &(&(&translation(0.0, 0.0, 5.0) * &rotation_y(FRAC_PI_4)) * &rotation_x(FRAC_PI_2))
            * &scaling(10.0, 0.01, 10.0),
    );
    right_wall.material = floor.material.clone();

    // Middle sphere
    let mut middle = Sphere::new();
    middle.set_transform(translation(-0.5, 1.0, 0.5));
    middle.material = Material::new();
    middle.material.color = Color::new(0.1, 1.0, 0.5);
    middle.material.diffuse = 0.7;
    middle.material.specular = 0.3;

    // Right sphere
    let mut right = Sphere::new();
    right.set_transform(&translation(1.5, 0.5, -0.5) * &scaling(0.5, 0.5, 0.5));
    right.material = Material::new();
    right.material.color = Color::new(0.5, 1.0, 0.1);
    right.material.diffuse = 0.7;
    right.material.specular = 0.3;

    // Left sphere
    let mut left = Sphere::new();
    left.set_transform(&translation(-1.5, 0.33, -0.75) * &scaling(0.33, 0.33, 0.33));
    left.material = Material::new();
    left.material.color = Color::new(1.0, 0.8, 0.1);
    left.material.diffuse = 0.7;
    left.material.specular = 0.3;

    let world = World {
        objects: vec![floor, left_wall, right_wall, middle, right, left],
        lights: vec![PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        )],
    };

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 1.5, -5.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/six_spheres_shadowed.ppm", ppm)
        .expect("Failed to write six_spheres_shadowed.ppm");
    println!("Saved to media/images_ppm/six_spheres_shadowed.ppm");
}
