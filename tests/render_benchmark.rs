//! Static render benchmark + regression test.
//!
//! Goal: track how long it takes to render a fixed scene as the renderer is
//! optimized step by step, while guaranteeing the output does not change.
//!
//! The test:
//!   1. Builds a fixed, deterministic scene (a single shaded sphere + one light).
//!   2. Times the render pipeline from "scene fired" (`camera.render`) up to the
//!      point the PPM string is created (`canvas_to_ppm`).
//!   3. Compares the generated PPM against a committed reference, pixel by pixel.
//!
//! Timing output is only printed by the test harness when a test fails or when
//! run with `--nocapture`, so measure with:
//!     cargo test --release --test render_benchmark -- --nocapture
//!
//! To (re)generate the reference image after an *intentional* change to the
//! rendered output, run:
//!     UPDATE_REFERENCE=1 cargo test --release --test render_benchmark

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::view_transform;
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;
use std::time::Instant;

/// Resolution of the benchmark render. Kept fixed so the reference image and
/// timing numbers stay comparable across runs.
const WIDTH: usize = 400;
const HEIGHT: usize = 200;

/// Path to the committed golden PPM (note: `*.ppm` is git-ignored, so this file
/// is force-added to the repo).
const REFERENCE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/benchmark_sphere.ppm"
);

/// Builds the fixed benchmark scene: one purple sphere lit by a single point
/// light, viewed head-on. Everything here is deterministic so the render is
/// byte-for-byte reproducible.
fn build_scene() -> (Camera, World) {
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

    let mut camera = Camera::new(WIDTH, HEIGHT, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 0.0, -5.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

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

#[test]
fn benchmark_sphere_render_matches_reference() {
    let (camera, world) = build_scene();

    // ---- timed region: fire the scene -> PPM is created ----
    let start = Instant::now();
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let elapsed = start.elapsed();
    // --------------------------------------------------------

    println!(
        "[benchmark] rendered {WIDTH}x{HEIGHT} sphere scene to PPM in {elapsed:.3?} \
         ({:.1} pixels/ms)",
        (WIDTH * HEIGHT) as f64 / elapsed.as_secs_f64() / 1000.0
    );

    if std::env::var_os("UPDATE_REFERENCE").is_some() {
        std::fs::create_dir_all(
            std::path::Path::new(REFERENCE_PATH)
                .parent()
                .expect("reference path has a parent dir"),
        )
        .expect("failed to create fixtures dir");
        std::fs::write(REFERENCE_PATH, &ppm).expect("failed to write reference PPM");
        println!("[benchmark] wrote reference PPM to {REFERENCE_PATH}");
        return;
    }

    let expected = std::fs::read_to_string(REFERENCE_PATH).unwrap_or_else(|_| {
        panic!(
            "reference PPM not found at {REFERENCE_PATH}; \
             generate it with `UPDATE_REFERENCE=1 cargo test --test render_benchmark`"
        )
    });

    if let Err(diff) = compare_ppm_pixels(&ppm, &expected) {
        panic!("rendered image does not match reference: {diff}");
    }
}
