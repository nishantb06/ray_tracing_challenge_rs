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
    // 1) Load OBJ text
    let path = "files/IronMan.obj";
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path, e));
    println!("OBJ file loaded: {} lines", src.lines().count());

    // 2) Parse OBJ, and log the time taken
    let mut parser = ObjParser::new();
    let start = std::time::Instant::now();
    parser.parse(&src);
    let duration = start.elapsed();
    println!("OBJ parsing took {:.3?}", duration);

    // 3) Convert to a Group
    let mut model = parser.obj_to_group();

    // Optional transform to position/scale it
    model.set_transform(
        &(&rotation_y(0.0) * &translation(0.0, 0.0, 0.0)) * &scaling(1.0, 1.0, 1.0),
    );

    // 4) Build a simple world
    let mut world = World::new();

    // Background
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.9, 0.95);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    world.add_shape(background);

    // Add the OBJ model
    world.add_shape(model);

    // Lights
    world.lights = vec![
        PointLight::new(
            Tuple::point(-8.0, 12.0, -4.0),
            Color::new(1.0, 0.98, 0.95),
        ),
    ];

    // Camera
    let mut camera = Camera::new(400, 300, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(3.0, 2.0, -6.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let out = "media/images_ppm/final_base_mesh.ppm";
    std::fs::write(out, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", out);
}