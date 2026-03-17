use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{
    CheckersPattern, GradientPattern, Pattern, RingPattern, StripePattern,
};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Floor: vivid yellow & deep purple 3D checkers
    let mut floor_pattern = CheckersPattern::new(
        Color::new(1.0, 0.9, 0.1),
        Color::new(0.2, 0.0, 0.3),
    );
    // Slightly scale the floor pattern for denser checkers
    floor_pattern.set_transform(scaling(0.5, 0.5, 0.5));

    let mut floor = Plane::new();
    floor.data.material = Material::new();
    floor.data.material.color = Color::new(1.0, 1.0, 1.0);
    floor.data.material.specular = 0.0;
    floor.data.material.pattern = Some(Box::new(floor_pattern));

    // Back wall: cyan → magenta horizontal gradient
    let mut back_grad = GradientPattern::new(
        Color::new(0.0, 1.0, 1.0),
        Color::new(1.0, 0.0, 1.0),
    );
    // Stretch gradient a bit
    back_grad.set_transform(scaling(2.0, 1.0, 1.0));

    let mut back_wall = Plane::new();
    back_wall.set_transform(translation(0.0, 0.0, 10.0));
    back_wall.data.material = Material::new();
    back_wall.data.material.color = Color::new(1.0, 1.0, 1.0);
    back_wall.data.material.specular = 0.0;
    back_wall.data.material.pattern = Some(Box::new(back_grad));

    // Side wall: ring pattern in red ↔ white
    let mut side_ring = RingPattern::new(
        Color::new(1.0, 0.2, 0.2),
        Color::new(1.0, 1.0, 1.0),
    );
    // Rotate rings so they read nicely from camera
    side_ring.set_transform(rotation_z(FRAC_PI_3));

    let mut side_wall = Plane::new();
    side_wall.set_transform(&translation(-7.0, 0.0, 0.0) * &rotation_y(FRAC_PI_3));
    side_wall.data.material = Material::new();
    side_wall.data.material.color = Color::new(1.0, 1.0, 1.0);
    side_wall.data.material.specular = 0.0;
    side_wall.data.material.pattern = Some(Box::new(side_ring));

    // Sphere 1: classic vivid stripes
    let mut stripe = StripePattern::new(
        Color::new(1.0, 0.0, 0.0),
        Color::new(0.0, 0.6, 1.0),
    );
    stripe.set_transform(rotation_y(FRAC_PI_3));

    let mut s1 = Sphere::new();
    s1.set_transform(translation(-1.5, 1.0, 1.0));
    s1.data.material = Material::new();
    s1.data.material.color = Color::new(1.0, 1.0, 1.0);
    s1.data.material.diffuse = 0.7;
    s1.data.material.specular = 0.3;
    s1.data.material.pattern = Some(Box::new(stripe));

    // Sphere 2: gradient sphere (orange to teal)
    let mut grad = GradientPattern::new(
        Color::new(1.0, 0.5, 0.0),
        Color::new(0.0, 0.9, 0.7),
    );
    grad.set_transform(&scaling(0.7, 0.7, 0.7) * &rotation_y(FRAC_PI_3));

    let mut s2 = Sphere::new();
    s2.set_transform(translation(0.0, 1.2, 0.0));
    s2.data.material = Material::new();
    s2.data.material.color = Color::new(1.0, 1.0, 1.0);
    s2.data.material.diffuse = 0.7;
    s2.data.material.specular = 0.3;
    s2.data.material.pattern = Some(Box::new(grad));

    // Sphere 3: ring pattern sphere (lime ↔ navy)
    let mut ring = RingPattern::new(
        Color::new(0.4, 1.0, 0.2),
        Color::new(0.0, 0.1, 0.4),
    );
    ring.set_transform(scaling(0.8, 0.8, 0.8));

    let mut s3 = Sphere::new();
    s3.set_transform(translation(1.8, 0.8, -0.5));
    s3.data.material = Material::new();
    s3.data.material.color = Color::new(1.0, 1.0, 1.0);
    s3.data.material.diffuse = 0.7;
    s3.data.material.specular = 0.3;
    s3.data.material.pattern = Some(Box::new(ring));

    // Sphere 4: 3D checkers (electric blue vs bright green)
    let mut checker = CheckersPattern::new(
        Color::new(0.2, 0.4, 1.0),
        Color::new(0.1, 0.9, 0.3),
    );
    checker.set_transform(scaling(1.5, 1.5, 1.5));

    let mut s4 = Sphere::new();
    s4.set_transform(&translation(0.5, 0.5, -2.0) * &scaling(0.6, 0.6, 0.6));
    s4.data.material = Material::new();
    s4.data.material.color = Color::new(1.0, 1.0, 1.0);
    s4.data.material.diffuse = 0.7;
    s4.data.material.specular = 0.3;
    s4.data.material.pattern = Some(Box::new(checker));

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(side_wall);
    world.add_shape(s1);
    world.add_shape(s2);
    world.add_shape(s3);
    world.add_shape(s4);
    world.lights = vec![
        PointLight::new(
            Tuple::point(-8.0, 10.0, -8.0),
            Color::new(1.0, 1.0, 1.0),
        ),
        PointLight::new(
            Tuple::point(6.0, 6.0, -4.0),
            Color::new(0.6, 0.6, 0.6),
        ),
    ];

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 2.0, -7.0),
        &Tuple::point(0.0, 1.0, 1.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/crazy_patterns.ppm", ppm)
        .expect("Failed to write crazy_patterns.ppm");
    println!("Saved to media/images_ppm/crazy_patterns.ppm");
}

