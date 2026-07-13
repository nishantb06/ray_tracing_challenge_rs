//! Static render benchmark + regression test.
//!
//! Goal: track how long it takes to render fixed scenes as the renderer is
//! optimized step by step, while guaranteeing the output does not change.
//!
//! Each test:
//!   1. Builds a fixed, deterministic scene.
//!   2. Times the render pipeline from "scene fired" (`camera.render`) up to the
//!      point the PPM string is created (`canvas_to_ppm`).
//!   3. Compares the generated PPM against a committed reference, pixel by pixel.
//!
//! Timing output is only printed by the test harness when a test fails or when
//! run with `--nocapture`, so measure with:
//!     cargo test --release --test render_benchmark -- --nocapture
//!
//! Individual tests:
//!     cargo test --release --test render_benchmark benchmark_sphere_render_matches_reference -- --nocapture
//!     cargo test --release --test render_benchmark benchmark_cover_scene_render_matches_reference -- --nocapture
//!     cargo test --release --test render_benchmark benchmark_group_hexagon_render_matches_reference -- --nocapture
//!
//! To (re)generate reference images after an *intentional* change to the
//! rendered output, run:
//!     UPDATE_REFERENCE=1 cargo test --release --test render_benchmark

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::matrix::Matrix;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, PI};
use std::time::Instant;

/// Resolution of the sphere benchmark render. Kept fixed so the reference image
/// and timing numbers stay comparable across runs.
const SPHERE_WIDTH: usize = 400;
const SPHERE_HEIGHT: usize = 200;

/// Resolution of the cover-scene benchmark (matches `src/bin/cover_scene.rs`).
const COVER_WIDTH: usize = 2000;
const COVER_HEIGHT: usize = 2000;

/// Resolution of the group-hexagon benchmark (matches `src/bin/group_hexagon.rs`).
const HEXAGON_WIDTH: usize = 1000;
const HEXAGON_HEIGHT: usize = 1000;

const SPHERE_REFERENCE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/benchmark_sphere.ppm"
);

const COVER_REFERENCE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/cover_scene.ppm"
);

const HEXAGON_REFERENCE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/group_hexagon.ppm"
);

/// Builds the fixed sphere benchmark scene: one purple sphere lit by a single
/// point light, viewed head-on. Everything here is deterministic so the render
/// is byte-for-byte reproducible.
fn build_sphere_scene() -> (Camera, World) {
    let mut sphere = Sphere::new();
    sphere.data.material = Material::new();
    sphere.data.material.color = Color::new(1.0, 0.2, 1.0);
    sphere.data.material.diffuse = 0.7;
    sphere.data.material.specular = 0.3;

    let mut world = World::new();
    world.add_shape(sphere);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(SPHERE_WIDTH, SPHERE_HEIGHT, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 0.0, -5.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    (camera, world)
}

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

fn make_cube(mat: Material, transform: Matrix) -> Cube {
    let mut c = Cube::new();
    c.set_transform(transform);
    *c.material_mut() = mat;
    c
}

/// Builds the book cover scene from `src/bin/cover_scene.rs`.
fn build_cover_scene() -> (Camera, World) {
    let mut world = World::new();

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

    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 0.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        blue_material(),
        &translation(8.5, 1.5, -0.5) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(0.0, 0.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 4.0) * &small_object(),
    ));
    world.add_shape(make_cube(
        purple_material(),
        &translation(7.5, 0.5, 4.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.25, 0.25, 8.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        blue_material(),
        &translation(4.0, 1.0, 7.5) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(10.0, 2.0, 7.5) * &medium_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(8.0, 2.0, 12.0) * &small_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(20.0, 1.0, 9.0) * &small_object(),
    ));
    world.add_shape(make_cube(
        blue_material(),
        &translation(-0.5, -5.0, 0.25) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(4.0, -4.0, 0.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(8.5, -4.0, 0.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -4.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        purple_material(),
        &translation(-0.5, -4.5, 8.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -8.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.5, -8.5, 8.0) * &large_object(),
    ));

    let mut camera = Camera::new(COVER_WIDTH, COVER_HEIGHT, 0.785);
    camera.set_transform(view_transform(
        &Tuple::point(-6.0, 6.0, -10.0),
        &Tuple::point(6.0, 0.0, 6.0),
        &Tuple::vector(-0.45, 1.0, 0.0),
    ));

    (camera, world)
}

const HEX_COLOR: Color = Color {
    red: 0.2,
    green: 0.8,
    blue: 1.0,
};

fn hexagon_corner() -> Sphere {
    let mut corner = Sphere::new();
    corner.set_transform(&translation(0.0, 0.0, -1.0) * &scaling(0.25, 0.25, 0.25));
    corner.material_mut().color = HEX_COLOR;
    corner
}

fn hexagon_edge() -> Cylinder {
    let mut edge = Cylinder::new();
    edge.minimum = 0.0;
    edge.maximum = 1.0;
    edge.set_transform(
        &(&(&translation(0.0, 0.0, -1.0) * &rotation_y(-PI / 6.0)) * &rotation_z(-FRAC_PI_2))
            * &scaling(0.25, 1.0, 0.25),
    );
    edge.material_mut().color = HEX_COLOR;
    edge
}

fn hexagon_side() -> Group {
    let mut side = Group::new();
    side.add_child(Box::new(hexagon_corner()));
    side.add_child(Box::new(hexagon_edge()));
    side
}

fn hexagon() -> Group {
    let mut hex = Group::new();
    for n in 0..6 {
        let mut side = hexagon_side();
        side.set_transform(rotation_y(n as f64 * PI / 3.0));
        hex.add_child(Box::new(side));
    }
    hex
}

/// Builds the group hexagon scene from `src/bin/group_hexagon.rs`.
fn build_group_hexagon_scene() -> (Camera, World) {
    let mut world = World::new();
    world.lights = vec![
        PointLight::new(Tuple::point(-10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(-10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
    ];
    world.add_shape(hexagon());

    let mut camera = Camera::new(HEXAGON_WIDTH, HEXAGON_HEIGHT, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(1.0, 1.25, -3.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    (camera, world)
}

/// Parses a P3 (ASCII) PPM into `(width, height, pixel_values)`, where
/// `pixel_values` is a flat list of the RGB integer channels in row-major order.
/// Whitespace/line-wrapping is irrelevant since we tokenize on whitespace.
fn parse_ppm(data: &str) -> (usize, usize, Vec<u32>) {
    let mut tokens = data.split_whitespace();
    assert_eq!(tokens.next(), Some("P3"), "reference/render is not a P3 PPM");
    let width = tokens
        .next()
        .and_then(|t| t.parse().ok())
        .expect("missing PPM width");
    let height = tokens
        .next()
        .and_then(|t| t.parse().ok())
        .expect("missing PPM height");
    let _max_color: u32 = tokens
        .next()
        .and_then(|t| t.parse().ok())
        .expect("missing PPM max color value");
    let values = tokens
        .map(|t| t.parse().expect("non-numeric channel in PPM"))
        .collect();
    (width, height, values)
}

/// Compares two PPM strings pixel by pixel and returns `Err` describing the
/// first difference found, or `Ok(())` if they match exactly.
fn compare_ppm_pixels(actual: &str, expected: &str) -> Result<(), String> {
    let (aw, ah, apx) = parse_ppm(actual);
    let (ew, eh, epx) = parse_ppm(expected);

    if (aw, ah) != (ew, eh) {
        return Err(format!(
            "dimension mismatch: rendered {aw}x{ah}, reference {ew}x{eh}"
        ));
    }
    if apx.len() != epx.len() {
        return Err(format!(
            "channel count mismatch: rendered {} values, reference {} values",
            apx.len(),
            epx.len()
        ));
    }

    for (i, (a, e)) in apx.iter().zip(epx.iter()).enumerate() {
        if a != e {
            let pixel = i / 3;
            let channel = ["R", "G", "B"][i % 3];
            let (x, y) = (pixel % aw, pixel / aw);
            return Err(format!(
                "pixel mismatch at (x={x}, y={y}) channel {channel}: rendered {a}, reference {e}"
            ));
        }
    }
    Ok(())
}

fn maybe_update_reference(path: &str, ppm: &str) -> bool {
    if std::env::var_os("UPDATE_REFERENCE").is_none() {
        return false;
    }
    std::fs::create_dir_all(
        std::path::Path::new(path)
            .parent()
            .expect("reference path has a parent dir"),
    )
    .expect("failed to create fixtures dir");
    std::fs::write(path, ppm).expect("failed to write reference PPM");
    println!("[benchmark] wrote reference PPM to {path}");
    true
}

fn assert_matches_reference(ppm: &str, reference_path: &str) {
    let expected = std::fs::read_to_string(reference_path).unwrap_or_else(|_| {
        panic!(
            "reference PPM not found at {reference_path}; \
             generate it with `UPDATE_REFERENCE=1 cargo test --test render_benchmark`"
        )
    });

    if let Err(diff) = compare_ppm_pixels(ppm, &expected) {
        panic!("rendered image does not match reference: {diff}");
    }
}

#[test]
fn benchmark_sphere_render_matches_reference() {
    let (camera, world) = build_sphere_scene();

    // ---- timed region: fire the scene -> PPM is created ----
    let start = Instant::now();
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let elapsed = start.elapsed();
    // --------------------------------------------------------

    println!(
        "[benchmark] rendered {SPHERE_WIDTH}x{SPHERE_HEIGHT} sphere scene to PPM in {elapsed:.3?} \
         ({:.1} pixels/ms)",
        (SPHERE_WIDTH * SPHERE_HEIGHT) as f64 / elapsed.as_secs_f64() / 1000.0
    );

    if maybe_update_reference(SPHERE_REFERENCE_PATH, &ppm) {
        return;
    }
    assert_matches_reference(&ppm, SPHERE_REFERENCE_PATH);
}

#[test]
fn benchmark_cover_scene_render_matches_reference() {
    let (camera, world) = build_cover_scene();

    // ---- timed region: fire the scene -> PPM is created ----
    let start = Instant::now();
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let elapsed = start.elapsed();
    // --------------------------------------------------------

    println!(
        "[benchmark] rendered {COVER_WIDTH}x{COVER_HEIGHT} cover scene to PPM in {elapsed:.3?} \
         ({:.1} pixels/ms)",
        (COVER_WIDTH * COVER_HEIGHT) as f64 / elapsed.as_secs_f64() / 1000.0
    );

    if maybe_update_reference(COVER_REFERENCE_PATH, &ppm) {
        return;
    }
    assert_matches_reference(&ppm, COVER_REFERENCE_PATH);
}

#[test]
fn benchmark_group_hexagon_render_matches_reference() {
    let (camera, world) = build_group_hexagon_scene();

    // ---- timed region: fire the scene -> PPM is created ----
    let start = Instant::now();
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let elapsed = start.elapsed();
    // --------------------------------------------------------

    println!(
        "[benchmark] rendered {HEXAGON_WIDTH}x{HEXAGON_HEIGHT} group hexagon scene to PPM in {elapsed:.3?} \
         ({:.1} pixels/ms)",
        (HEXAGON_WIDTH * HEXAGON_HEIGHT) as f64 / elapsed.as_secs_f64() / 1000.0
    );

    if maybe_update_reference(HEXAGON_REFERENCE_PATH, &ppm) {
        return;
    }
    assert_matches_reference(&ppm, HEXAGON_REFERENCE_PATH);
}
