// A sphere partially embedded in the floor plane (half above, half below).
use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Floor plane (y = 0)
    let mut floor = Plane::new();
    floor.data.material.color = Color::new(1.0, 0.9, 0.9);
    floor.data.material.specular = 0.0;

    // Sphere with center at (0, 0.6, 0): radius 1, so it sits from y = -0.4 to y = 1.6 (mostly above)
    // For half embedded use center at (0, 0.5, 0) so equator is at y=0.
    let mut embedded = Sphere::new();
    embedded.set_transform(translation(0.0, 0.5, 0.0));
    embedded.data.material = Material::new();
    embedded.data.material.color = Color::new(0.2, 0.4, 0.9);
    embedded.data.material.diffuse = 0.7;
    embedded.data.material.specular = 0.3;

    // Another sphere fully on the floor for comparison
    let mut on_floor = Sphere::new();
    on_floor.set_transform(translation(1.8, 1.0, 0.2));
    on_floor.data.material = Material::new();
    on_floor.data.material.color = Color::new(1.0, 0.5, 0.2);
    on_floor.data.material.diffuse = 0.7;
    on_floor.data.material.specular = 0.3;

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(embedded);
    world.add_shape(on_floor);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(1000, 500, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 1.2, -4.0),
        &Tuple::point(0.0, 0.5, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/embedded_sphere.ppm", ppm)
        .expect("Failed to write embedded_sphere.ppm");
    println!("Saved to media/images_ppm/embedded_sphere.ppm");
}
