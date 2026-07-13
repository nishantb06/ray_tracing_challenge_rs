use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::transformation::{rotation_x, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;

fn main() {
    // ======================================================
    // materials (matching the YAML defines)
    // ======================================================
    let white_material = Material {
        color: Color::new(1.0, 1.0, 1.0),
        diffuse: 0.7,
        ambient: 0.1,
        specular: 0.0,
        reflective: 0.1,
        ..Material::new()
    };

    let blue_material = Material {
        color: Color::new(0.537, 0.831, 0.914),
        ..white_material
    };

    // ======================================================
    // camera
    // ======================================================
    let mut camera = Camera::new(100, 100, 0.785);
    camera.set_transform(view_transform(
        &Tuple::point(-6.0, 6.0, -10.0),
        &Tuple::point(6.0, 0.0, 6.0),
        &Tuple::vector(-0.45, 1.0, 0.0),
    ));

    // ======================================================
    // world + light sources
    // ======================================================
    let mut world = World::new();
    world.lights = vec![
        PointLight::new(Tuple::point(50.0, 100.0, -50.0), Color::new(1.0, 1.0, 1.0)),
        PointLight::new(
            Tuple::point(-400.0, 50.0, -10.0),
            Color::new(0.2, 0.2, 0.2),
        ),
    ];

    // ======================================================
    // a white backdrop for the scene
    // ======================================================
    let mut backdrop = Plane::new();
    {
        let m = backdrop.material_mut();
        *m = Material::new();
        m.color = Color::new(1.0, 1.0, 1.0);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    backdrop.set_transform(&translation(0.0, 0.0, 500.0) * &rotation_x(std::f64::consts::FRAC_PI_2));
    world.add_shape(backdrop);

    // ======================================================
    // cube
    // ======================================================
    // "large-object" from RTC YAML scenes: scaling(3.5, 3.5, 3.5)
    let large_object = scaling(3.5, 3.5, 3.5);

    let mut cube = Cube::new();
    cube.set_transform(&translation(8.5, 1.5, -0.5) * &large_object);
    *cube.material_mut() = blue_material;
    world.add_shape(cube);

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/complex_cube.ppm", ppm)
        .expect("Failed to write media/images_ppm/complex_cube.ppm");
    println!("Saved to media/images_ppm/complex_cube.ppm");
}

