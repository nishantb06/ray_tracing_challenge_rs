use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{GradientPattern, Pattern};
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Unit sphere with a red -> yellow gradient across its width.
    let mut grad_pattern = GradientPattern::new(
        Color::new(1.0, 0.0, 0.0), // red
        Color::new(1.0, 1.0, 0.0), // yellow
    );

    // Map sphere's local x in [-1, 1] to pattern x in [0, 1] so gradient
    // covers the sphere once from left (red) to right (yellow).
    //
    // We want pattern_x = (x + 1) / 2. The pattern code does:
    //   pattern_point = transform_inverse * object_point
    // so choose transform whose inverse applies that mapping.
    let transform = &scaling(2.0, 1.0, 1.0) * &translation(-0.5, 0.0, 0.0);
    grad_pattern.set_transform(transform);

    let mut sphere = Sphere::new();
    sphere.data.material = Material::new();
    sphere.data.material.color = Color::new(1.0, 1.0, 1.0);
    sphere.data.material.diffuse = 0.7;
    sphere.data.material.specular = 0.3;
    sphere.data.material.pattern = Some(Box::new(grad_pattern));

    let mut world = World::new();
    world.add_shape(sphere);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(800, 400, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 0.0, -5.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/gradient_sphere.ppm", ppm)
        .expect("Failed to write gradient_sphere.ppm");
    println!("Saved to media/images_ppm/gradient_sphere.ppm");
}

