//! Render the Utah teapot from `files/teapot.obj`.

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::obj_file::{obj_to_group, parse_obj_file_with_material};
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_y, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    let path = format!("{}/files/teapot.obj", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path, e));

    let mut porcelain = Material::new();
    porcelain.color = Color::new(0.90, 0.86, 0.78);
    porcelain.ambient = 0.08;
    porcelain.diffuse = 0.75;
    porcelain.specular = 0.45;
    porcelain.shininess = 96.0;

    let parser: &'static _ = Box::leak(Box::new(parse_obj_file_with_material(&src, &porcelain)));

    let mut teapot = obj_to_group(parser);
    // Center roughly in X/Z, lift to sit near origin; Utah teapot spans ~[-3, 3] in X in file space.
    teapot.set_transform(
        &( &rotation_y(-0.35) * &translation(0.15, -1.2, 0.0) ) * &scaling(0.42, 0.42, 0.42),
    );

    let mut world = World::new();

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

    world.add_shape(teapot);

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

    // ~6k triangles: 400×300 is a practical default; raise to 800×600 for final quality (slower).
    let mut camera = Camera::new(400, 300, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(5.0, 3.2, -6.0),
        &Tuple::point(0.0, 0.15, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let out = "media/images_ppm/teapot.ppm";
    std::fs::write(out, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", out);
}
