use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, rotation_z, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Reflective checkerboard floor
    let mut floor_pattern = CheckersPattern::new(
        Color::new(1.0, 1.0, 1.0), // white
        Color::new(0.0, 0.0, 0.0), // black
    );
    // Slightly larger checks so the pattern reads cleanly
    floor_pattern.set_transform(scaling(1.5, 1.0, 1.5));

    let mut floor = Plane::new();
    floor.data.material = Material::new();
    floor.data.material.ambient = 0.2;
    floor.data.material.diffuse = 0.7;
    floor.data.material.specular = 0.1;
    floor.data.material.pattern = Some(Box::new(floor_pattern));
    // Make the floor reflective
    floor.data.material.reflective = 0.6;

    // Simple red ball resting on the floor
    let mut ball = Sphere::new();
    ball.set_transform(translation(0.0, 1.0, 0.0));
    ball.data.material = Material::new();
    ball.data.material.color = Color::new(1.0, 0.1, 0.1);
    ball.data.material.ambient = 0.1;
    ball.data.material.diffuse = 0.7;
    ball.data.material.specular = 0.3;
    ball.data.material.shininess = 100.0;

    // Vertical back wall (YZ-ish), behind the ball
    let mut back_wall = Plane::new();
    back_wall.set_transform(
        &translation(0.0, 0.0, 8.0) * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    back_wall.data.material = Material::new();
    back_wall.data.material.color = Color::new(0.9, 0.9, 0.9);
    back_wall.data.material.ambient = 0.1;
    back_wall.data.material.diffuse = 0.7;
    back_wall.data.material.specular = 0.0;

    // Vertical side wall (XZ-ish), to the right of the ball
    let mut side_wall = Plane::new();
    side_wall.set_transform(
        &(&translation(6.0, 0.0, 0.0) * &rotation_z(std::f64::consts::FRAC_PI_2))
            * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    side_wall.data.material = Material::new();
    side_wall.data.material.color = Color::new(0.9, 0.9, 0.9);
    side_wall.data.material.ambient = 0.1;
    side_wall.data.material.diffuse = 0.7;
    side_wall.data.material.specular = 0.0;

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(side_wall);
    world.add_shape(ball);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // Move camera slightly farther back so ball and its reflection fit comfortably
    let mut camera = Camera::new(3840, 2160, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 2.0, -9.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/reflective_floor.ppm", ppm)
        .expect("Failed to write reflective_floor.ppm");
    println!("Saved to media/images_ppm/reflective_floor.ppm");
}

