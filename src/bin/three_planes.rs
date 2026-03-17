use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, GradientPattern, Pattern, RingPattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};

fn main() {
    // Floor plane with vivid 3D checkers (quadrant base)
    let mut floor_pattern = CheckersPattern::new(
        Color::new(0.9, 0.9, 0.2),
        Color::new(0.1, 0.1, 0.4),
    );
    floor_pattern.set_transform(scaling(0.7, 0.7, 0.7));

    let mut floor = Plane::new();
    floor.data.material = Material::new();
    floor.data.material.color = Color::new(1.0, 1.0, 1.0);
    floor.data.material.ambient = 0.3;
    floor.data.material.diffuse = 0.8;
    floor.data.material.specular = 0.0;
    floor.data.material.pattern = Some(Box::new(floor_pattern));

    // Back wall with horizontal gradient (cyan -> magenta)
    let mut back_grad = GradientPattern::new(
        Color::new(0.0, 1.0, 1.0),
        Color::new(1.0, 0.0, 1.0),
    );
    back_grad.set_transform(scaling(2.0, 1.0, 1.0));

    let mut back_wall = Plane::new();
    // Rotate up into vertical plane, then push back in +z
    back_wall.set_transform(&translation(0.0, 0.0, 8.0) * &rotation_x(FRAC_PI_2));
    back_wall.data.material = Material::new();
    back_wall.data.material.color = Color::new(1.0, 1.0, 1.0);
    back_wall.data.material.ambient = 0.3;
    back_wall.data.material.diffuse = 0.8;
    back_wall.data.material.specular = 0.0;
    back_wall.data.material.pattern = Some(Box::new(back_grad));

    // Side wall with concentric rings (red ↔ white)
    let mut side_ring = RingPattern::new(
        Color::new(1.0, 0.3, 0.3),
        Color::new(1.0, 1.0, 1.0),
    );
    side_ring.set_transform(scaling(0.8, 0.8, 0.8));

    let mut side_wall = Plane::new();
    // Start as vertical plane, rotate around y to form quadrant side
    let side_transform =
        &translation(-6.0, 0.0, 2.0) * &(&rotation_y(FRAC_PI_2) * &rotation_x(FRAC_PI_2));
    side_wall.set_transform(side_transform);
    side_wall.data.material = Material::new();
    side_wall.data.material.color = Color::new(1.0, 1.0, 1.0);
    side_wall.data.material.ambient = 0.3;
    side_wall.data.material.diffuse = 0.8;
    side_wall.data.material.specular = 0.0;
    side_wall.data.material.pattern = Some(Box::new(side_ring));

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(side_wall);
    world.lights = vec![
        PointLight::new(
            Tuple::point(-8.0, 8.0, -8.0),
            Color::new(1.0, 1.0, 1.0),
        ),
        PointLight::new(
            Tuple::point(5.0, 6.0, -3.0),
            Color::new(0.6, 0.6, 0.6),
        ),
    ];

    let mut camera = Camera::new(800, 400, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 2.0, -7.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/three_planes.ppm", ppm)
        .expect("Failed to write three_planes.ppm");
    println!("Saved to media/images_ppm/three_planes.ppm");
}

