# Ray Tracing Challenge in Rust

Rust implementation of *The Ray Tracer Challenge* by Jamis Buck, built from first principles:
tuples, matrices, transformations, rays, intersections, materials, lights, patterns, reflections,
refractions, and multi-object worlds.

![Cover scene render](media/images/cover_scene.png)

## Why this project is worth checking out

- Built without a graphics engine: core ray tracing math and rendering pipeline are implemented directly in Rust.
- Progresses from fundamentals to full scenes: starts with simple primitives and ends with complex compositions.
- Supports physically inspired effects: shadows, reflections, transparency, and refraction.
- Includes a rich scene gallery: 25+ binary examples under `src/bin/` for experimenting with materials and patterns.
- Keeps code modular: rendering primitives and math utilities are separated into reusable modules.

## Implemented features

- **Math core**: tuples/vectors, matrices, transformations, view transforms.
- **Geometry**: sphere, plane, cube.
- **Ray logic**: ray-object intersections, hit computation, world intersection management.
- **Shading model**: Phong lighting (ambient, diffuse, specular), shadows.
- **Materials**: reflectivity, transparency, refractive index.
- **Patterns**: stripe, gradient, ring, checker (with object and pattern transforms).
- **Camera + rendering**: ray generation per pixel and full scene render to PPM.

## Quick start

### 1) Build

```bash
cargo build
```

### 2) Render a scene

Pick any binary from `src/bin/`. For example:

```bash
cargo run --bin cover_scene
```

This writes a PPM file (for `cover_scene`, output is `media/images_ppm/cover_scene.ppm`).

### 3) Convert PPM to PNG

```bash
cargo run --bin ppm_to_png -- media/images_ppm/cover_scene.ppm media/images/cover_scene.png
```

## Scene gallery

| Scene | Preview |
| --- | --- |
| Cover scene | ![Cover scene](media/images/cover_scene.png) |
| Single glass sphere | ![Single glass sphere](media/images/single_glass_sphere.png) |
| Reflective floor | ![Reflective floor](media/images/reflective_floor.png) |
| Pattern scene | ![Pattern scene](media/images/pattern_scene.png) |
| Checker sphere | ![Checker sphere](media/images/checkered_sphere.png) |
| Checker plane | ![Checker plane](media/images/checkered_plane.png) |
| Wall backdrop | ![Wall backdrop](media/images/wall_backdrop.png) |
| Ceiling | ![Ceiling](media/images/ceiling.png) |
| Embedded sphere | ![Embedded sphere](media/images/embedded_sphere.png) |

Additional renders are available in `media/images/`.

## Project structure

```text
src/
  lib.rs              # Library exports
  tuple.rs            # Points, vectors, vector ops
  matrix.rs           # Matrix math and inversion
  transformation.rs   # Translation, scaling, rotation, view transforms
  ray.rs              # Rays and ray transforms
  intersection.rs     # Intersections and hit logic
  material.rs         # Material properties
  light.rs            # Point lights
  pattern.rs          # Procedural patterns
  sphere.rs
  plane.rs
  cube.rs
  world.rs            # Scene graph + shading pipeline
  camera.rs           # Camera model + rendering
  canvas.rs           # Pixel buffer + PPM export
  bin/                # Scene entrypoints (25+ examples)
```

## Notable scene entrypoints

- `cover_scene`
- `single_glass_sphere`
- `reflective_floor`
- `pattern_scene`
- `complex_cube`
- `hexagonal_room`
- `solar_system`

## Tech stack

- Rust Edition 2024
- Dependency: [`image`](https://crates.io/crates/image) for image conversion workflows

## Reference

- Book: [The Ray Tracer Challenge](https://pragprog.com/titles/jbtracer/the-ray-tracer-challenge/)