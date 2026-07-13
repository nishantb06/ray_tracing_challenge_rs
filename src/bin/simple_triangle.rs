use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{scaling, view_transform};
use ray_tracing_challenge_rs::triangle::Triangle;
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use ray_tracing_challenge_rs::shape::Shape;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Large matte sphere so rays that miss the triangle get a light background.
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.95, 0.95, 0.98);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }

    // Triangle in object space; default corners match the book (lies in plane z = 0).
    let mut tri = Triangle::new(
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::point(-1.0, 0.0, 0.0),
        Tuple::point(1.0, 0.0, 0.0),
    );
    {
        let m = tri.material_mut();
        *m = Material::new();
        m.color = Color::new(0.15, 0.55, 0.95);
        m.ambient = 0.1;
        m.diffuse = 0.85;
        m.specular = 0.3;
        m.shininess = 50.0;
    }

    let mut world = World::new();
    world.add_shape(background);
    world.add_shape(tri);
    world.lights = vec![PointLight::new(
        Tuple::point(-4.0, 6.0, -4.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // Camera in front of the triangle (negative z), looking at its center.
    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(1.0, 0.65, -3.25),
        &Tuple::point(0.0, 0.35, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let path = "media/images_ppm/simple_triangle.ppm";
    std::fs::write(path, canvas.canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", path);
}