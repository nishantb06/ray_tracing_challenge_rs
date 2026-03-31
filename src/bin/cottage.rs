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
    let path = format!(
        "{}/files/85-cottage_obj/cottage_obj.obj",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path, e));
    println!("OBJ file loaded from {} ({} lines)", path, src.lines().count());

    // 2) Parse OBJ
    let mut parser = ObjParser::new();
    parser.parse(&src);

    // 3) Convert to a Group
    let mut cottage = parser.obj_to_group();

    // Rough positioning/scaling; tweak as needed.
    cottage.set_transform(
        &(&rotation_y(0.3) * &translation(0.0, 0.0, 0.0)) * &scaling(0.5, 0.5, 0.5),
    );

    // 4) Build a simple world
    let mut world = World::new();

    // Background sphere so rays that miss see a light backdrop.
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.92, 0.94, 0.98);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    world.add_shape(background);

    // Add the OBJ model
    world.add_shape(cottage);

    // Lights
    world.lights = vec![
        PointLight::new(
            Tuple::point(-8.0, 12.0, -8.0),
            Color::new(1.0, 0.98, 0.95),
        ),
        PointLight::new(
            Tuple::point(6.0, 4.0, -6.0),
            Color::new(0.4, 0.45, 0.6),
        ),
    ];

    // Camera
    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(16.0, 14.0, -15.0),
        &Tuple::point(0.0, 1.5, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let out = "media/images_ppm/cottage.ppm";
    std::fs::write(out, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", out);
}

