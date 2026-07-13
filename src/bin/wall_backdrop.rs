// Three spheres on a floor with a wall as backdrop: plane rotated π/2 around x, translated in +z.
use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Floor plane
    let mut floor = Plane::new();
    floor.data.material.color = Color::new(1.0, 0.9, 0.9);
    floor.data.material.specular = 0.0;

    // Back wall: rotate π/2 around x so it stands vertical (normal +z), then translate in +z
    let mut wall = Plane::new();
    wall.set_transform(&translation(0.0, 0.0, 10.0) * &rotation_x(FRAC_PI_2));
    wall.data.material.color = Color::new(0.85, 0.85, 0.95);
    wall.data.material.specular = 0.0;

    // Middle sphere
    let mut middle = Sphere::new();
    middle.set_transform(translation(-0.5, 1.0, 0.5));
    middle.data.material = Material::new();
    middle.data.material.color = Color::new(0.1, 1.0, 0.5);
    middle.data.material.diffuse = 0.7;
    middle.data.material.specular = 0.3;

    // Right sphere
    let mut right = Sphere::new();
    right.set_transform(&translation(1.5, 0.5, -0.5) * &scaling(0.5, 0.5, 0.5));
    right.data.material = Material::new();
    right.data.material.color = Color::new(0.5, 1.0, 0.1);
    right.data.material.diffuse = 0.7;
    right.data.material.specular = 0.3;

    // Left sphere
    let mut left = Sphere::new();
    left.set_transform(&translation(-1.5, 0.33, -0.75) * &scaling(0.33, 0.33, 0.33));
    left.data.material = Material::new();
    left.data.material.color = Color::new(1.0, 0.8, 0.1);
    left.data.material.diffuse = 0.7;
    left.data.material.specular = 0.3;

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(wall);
    world.add_shape(middle);
    world.add_shape(right);
    world.add_shape(left);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 1.5, -5.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/wall_backdrop.ppm", ppm).expect("Failed to write wall_backdrop.ppm");
    println!("Saved to media/images_ppm/wall_backdrop.ppm");
}
