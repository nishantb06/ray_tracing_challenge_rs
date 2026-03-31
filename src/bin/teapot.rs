//! Render the Utah teapot from `files/teapot.obj` using `ObjParser`.

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::obj_parser::ObjParser;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_y, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    let path = format!("{}/files/teapot.obj", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path, e));

    // Parse OBJ
    let mut parser = ObjParser::new();
    parser.parse(&src);

    // Convert parsed model to a renderable Group
    let mut teapot = parser.obj_to_group();

    // Position/scale (same idea as your older example)
    teapot.set_transform(
        &(&rotation_y(-0.35) * &translation(0.15, -1.2, 0.0)) * &scaling(0.42, 0.42, 0.42),
    );

    // Note: setting material on the Group itself does NOT automatically apply
    // to leaf triangles in your current design. This is just a placeholder.
    {
        let m = teapot.material_mut();
        *m = Material::new();
        m.color = Color::new(0.90, 0.86, 0.78);
        m.ambient = 0.08;
        m.diffuse = 0.75;
        m.specular = 0.45;
        m.shininess = 96.0;
    }

    let mut world = World::new();

    // Background sphere
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.88, 0.91, 0.96);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    world.add_shape(background);

    // Add teapot
    world.add_shape(teapot);

    // Lights
    world.lights = vec![
        PointLight::new(
            Tuple::point(-8.0, 12.0, -4.0),
            Color::new(1.0, 0.98, 0.95),
        ),
        PointLight::new(
            Tuple::point(6.0, 4.0, -8.0),
            Color::new(0.35, 0.4, 0.55),
        ),
    ];

    // Camera
    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(5.0, 3.2, -6.0),
        &Tuple::point(0.0, 0.15, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let out = "media/images_ppm/teapot.ppm";
    std::fs::write(out, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", out);
}