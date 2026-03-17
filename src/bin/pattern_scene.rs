use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{GradientPattern, Pattern, RingPattern, StripePattern, CheckersPattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, rotation_y, rotation_z, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Floor: neutral checker pattern
    let mut floor_pattern = CheckersPattern::new(
        Color::new(0.9, 0.9, 0.9),
        Color::new(0.2, 0.2, 0.2),
    );
    // Use larger checks to avoid aliasing \"noise\" in the distance
    floor_pattern.set_transform(scaling(2.0, 2.0, 2.0));

    let mut floor = Plane::new();
    floor.data.material = Material::new();
    floor.data.material.color = Color::new(1.0, 1.0, 1.0);
    floor.data.material.ambient = 0.2;
    floor.data.material.diffuse = 0.8;
    floor.data.material.specular = 0.0;
    floor.data.material.pattern = Some(Box::new(floor_pattern));

    // Back wall: diagonal stripes in two grays
    let mut wall_stripes = StripePattern::new(
        Color::new(0.8, 0.8, 0.8),
        Color::new(0.3, 0.3, 0.3),
    );
    // Rotate stripes to be diagonal and scale so they are not too thin
    wall_stripes.set_transform(&scaling(0.7, 0.7, 0.7) * &rotation_z(FRAC_PI_3));

    let mut back_wall = Plane::new();
    back_wall.set_transform(&translation(0.0, 0.0, 10.0) * &rotation_x(std::f64::consts::FRAC_PI_2));
    back_wall.data.material = Material::new();
    back_wall.data.material.color = Color::new(1.0, 1.0, 1.0);
    back_wall.data.material.ambient = 0.2;
    back_wall.data.material.diffuse = 0.8;
    back_wall.data.material.specular = 0.0;
    back_wall.data.material.pattern = Some(Box::new(wall_stripes));

    // Large sphere: green ring pattern
    let mut ring_pattern = RingPattern::new(
        Color::new(0.2, 0.9, 0.2),
        Color::new(0.0, 0.45, 0.0),
    );
    // Slight scale to keep clear, broad bands
    ring_pattern.set_transform(scaling(0.9, 0.9, 0.9));

    let mut big_sphere = Sphere::new();
    big_sphere.set_transform(&translation(-1.2, 1.0, 0.5) * &scaling(1.2, 1.2, 1.2));
    big_sphere.data.material = Material::new();
    big_sphere.data.material.color = Color::new(0.1, 0.8, 0.2);
    big_sphere.data.material.diffuse = 0.7;
    big_sphere.data.material.specular = 0.3;
    big_sphere.data.material.pattern = Some(Box::new(ring_pattern));

    // Small sphere: red→yellow gradient
    let mut grad_pattern = GradientPattern::new(
        Color::new(1.0, 0.1, 0.1),
        Color::new(1.0, 0.9, 0.0),
    );
    // Leave gradient mostly horizontal across the sphere
    grad_pattern.set_transform(scaling(1.0, 1.0, 1.0));

    let mut small_sphere = Sphere::new();
    small_sphere.set_transform(&translation(1.0, 0.5, -0.5) * &scaling(0.6, 0.6, 0.6));
    small_sphere.data.material = Material::new();
    small_sphere.data.material.color = Color::new(1.0, 0.9, 0.0);
    small_sphere.data.material.diffuse = 0.7;
    small_sphere.data.material.specular = 0.3;
    small_sphere.data.material.pattern = Some(Box::new(grad_pattern));

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(big_sphere);
    world.add_shape(small_sphere);
    world.lights = vec![
        PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        ),
        PointLight::new(
            Tuple::point(5.0, 8.0, -4.0),
            Color::new(0.5, 0.5, 0.5),
        ),
    ];

    // Narrower FOV and closer camera to show less distant floor (reduces aliasing)
    let mut camera = Camera::new(800, 400, FRAC_PI_3 / 1.8);
    camera.transform = view_transform(
        &Tuple::point(0.0, 1.4, -4.5),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/pattern_scene.ppm", ppm)
        .expect("Failed to write pattern_scene.ppm");
    println!("Saved to media/images_ppm/pattern_scene.ppm");
}

