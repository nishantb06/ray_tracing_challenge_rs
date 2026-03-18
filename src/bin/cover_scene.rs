use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::matrix::Matrix;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;

// ======================================================
// material helpers
// ======================================================

fn white_material() -> Material {
    Material {
        color: Color::new(1.0, 1.0, 1.0),
        diffuse: 0.7,
        ambient: 0.1,
        specular: 0.0,
        reflective: 0.1,
        ..Material::new()
    }
}

fn blue_material() -> Material {
    Material {
        color: Color::new(0.537, 0.831, 0.914),
        ..white_material()
    }
}

fn red_material() -> Material {
    Material {
        color: Color::new(0.941, 0.322, 0.388),
        ..white_material()
    }
}

fn purple_material() -> Material {
    Material {
        color: Color::new(0.373, 0.404, 0.550),
        ..white_material()
    }
}

// ======================================================
// transform helpers
//
// YAML convention: transforms listed top-to-bottom are
// innermost-first.  [T1, T2] → M = T2 * T1
//
// standard-transform = scale(0.5) * translate(1,-1,1)
// large-object       = scale(3.5) * standard
// medium-object      = scale(3)   * standard
// small-object       = scale(2)   * standard
// ======================================================

fn standard_transform() -> Matrix {
    &scaling(0.5, 0.5, 0.5) * &translation(1.0, -1.0, 1.0)
}

fn large_object() -> Matrix {
    &scaling(3.5, 3.5, 3.5) * &standard_transform()
}

fn medium_object() -> Matrix {
    &scaling(3.0, 3.0, 3.0) * &standard_transform()
}

fn small_object() -> Matrix {
    &scaling(2.0, 2.0, 2.0) * &standard_transform()
}

// ======================================================
// helper: create a cube with a given material and transform
// ======================================================

fn make_cube(mat: Material, transform: Matrix) -> Cube {
    let mut c = Cube::new();
    c.set_transform(transform);
    *c.material_mut() = mat;
    c
}

fn main() {
    let mut world = World::new();

    // ======================================================
    // light sources
    // ======================================================
    world.lights = vec![
        PointLight::new(
            Tuple::point(50.0, 100.0, -50.0),
            Color::new(1.0, 1.0, 1.0),
        ),
        PointLight::new(
            Tuple::point(-400.0, 50.0, -10.0),
            Color::new(0.2, 0.2, 0.2),
        ),
    ];

    // ======================================================
    // white backdrop plane
    // ======================================================
    let mut backdrop = Plane::new();
    {
        let m = backdrop.material_mut();
        m.color = Color::new(1.0, 1.0, 1.0);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    backdrop.set_transform(
        &translation(0.0, 0.0, 500.0) * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    world.add_shape(backdrop);

    // ======================================================
    // glass-like sphere
    // ======================================================
    let mut glass = Sphere::new();
    {
        let m = glass.material_mut();
        m.color = Color::new(0.373, 0.404, 0.550);
        m.diffuse = 0.2;
        m.ambient = 0.0;
        m.specular = 1.0;
        m.shininess = 200.0;
        m.reflective = 0.7;
        m.transparency = 0.7;
        m.refractive_index = 1.5;
    }
    glass.set_transform(large_object());
    world.add_shape(glass);

    // ======================================================
    // cubes (upper / visible layer)
    // ======================================================

    // white cube – medium, translate(4, 0, 0)
    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 0.0) * &medium_object(),
    ));

    // blue cube – large, translate(8.5, 1.5, -0.5)
    world.add_shape(make_cube(
        blue_material(),
        &translation(8.5, 1.5, -0.5) * &large_object(),
    ));

    // red cube – large, translate(0, 0, 4)
    world.add_shape(make_cube(
        red_material(),
        &translation(0.0, 0.0, 4.0) * &large_object(),
    ));

    // white cube – small, translate(4, 0, 4)
    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 4.0) * &small_object(),
    ));

    // purple cube – medium, translate(7.5, 0.5, 4)
    world.add_shape(make_cube(
        purple_material(),
        &translation(7.5, 0.5, 4.0) * &medium_object(),
    ));

    // white cube – medium, translate(-0.25, 0.25, 8)
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.25, 0.25, 8.0) * &medium_object(),
    ));

    // blue cube – large, translate(4, 1, 7.5)
    world.add_shape(make_cube(
        blue_material(),
        &translation(4.0, 1.0, 7.5) * &large_object(),
    ));

    // red cube – medium, translate(10, 2, 7.5)
    world.add_shape(make_cube(
        red_material(),
        &translation(10.0, 2.0, 7.5) * &medium_object(),
    ));

    // white cube – small, translate(8, 2, 12)
    world.add_shape(make_cube(
        white_material(),
        &translation(8.0, 2.0, 12.0) * &small_object(),
    ));

    // white cube – small, translate(20, 1, 9)
    world.add_shape(make_cube(
        white_material(),
        &translation(20.0, 1.0, 9.0) * &small_object(),
    ));

    // ======================================================
    // cubes (lower / underground layer)
    // ======================================================

    // blue cube – large, translate(-0.5, -5, 0.25)
    world.add_shape(make_cube(
        blue_material(),
        &translation(-0.5, -5.0, 0.25) * &large_object(),
    ));

    // red cube – large, translate(4, -4, 0)
    world.add_shape(make_cube(
        red_material(),
        &translation(4.0, -4.0, 0.0) * &large_object(),
    ));

    // white cube – large, translate(8.5, -4, 0)
    world.add_shape(make_cube(
        white_material(),
        &translation(8.5, -4.0, 0.0) * &large_object(),
    ));

    // white cube – large, translate(0, -4, 4)
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -4.0, 4.0) * &large_object(),
    ));

    // purple cube – large, translate(-0.5, -4.5, 8)
    world.add_shape(make_cube(
        purple_material(),
        &translation(-0.5, -4.5, 8.0) * &large_object(),
    ));

    // white cube – large, translate(0, -8, 4)
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -8.0, 4.0) * &large_object(),
    ));

    // white cube – large, translate(-0.5, -8.5, 8)
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.5, -8.5, 8.0) * &large_object(),
    ));

    // ======================================================
    // camera – 1000×1000 (bumped from 100×100 in the YAML)
    // ======================================================
    let mut camera = Camera::new(2000, 2000, 0.785);
    camera.transform = view_transform(
        &Tuple::point(-6.0, 6.0, -10.0),
        &Tuple::point(6.0, 0.0, 6.0),
        &Tuple::vector(-0.45, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/cover_scene.ppm", ppm)
        .expect("Failed to write media/images_ppm/cover_scene.ppm");
    println!("Saved to media/images_ppm/cover_scene.ppm");
}
