use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::csg::{CSG, CSGOperation};
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::matrix::Matrix;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

/// Half-size of the die in object space after `scaling(half, half, half)` (unit cube is ±1).
const DIE_HALF: f64 = 0.65;
/// Tiny gap so stacked dice do not share the exact same surface (reduces overlap / z-fighting).
const STACK_GAP: f64 = 0.02;

fn rot_scale_matrix(yaw: f64, pitch: f64, roll: f64, half: f64) -> Matrix {
    &(&(&rotation_y(yaw) * &rotation_x(pitch)) * &rotation_z(roll)) * &scaling(half, half, half)
}

/// Min/max world Y of the eight corners of the unit cube (±1,±1,±1) after `rs` (rotation + scale only).
fn cube_y_extents_after_rot_scale(rs: &Matrix) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for sx in [-1.0_f64, 1.0] {
        for sy in [-1.0_f64, 1.0] {
            for sz in [-1.0_f64, 1.0] {
                let p = Tuple::point(sx, sy, sz);
                let w = rs * &p;
                lo = lo.min(w.y);
                hi = hi.max(w.y);
            }
        }
    }
    (lo, hi)
}

/// Full die transform: `translation * rotation_y * rotation_x * rotation_z * scaling`.
fn die_transform(tx: f64, ty: f64, tz: f64, rs: &Matrix) -> Matrix {
    &translation(tx, ty, tz) * rs
}
 
fn pip_uv_for_number(n: u8, o: f64) -> Vec<(f64, f64)> {
    // Standard pip patterns on a face in (u,v) coordinates.
    // u/v are tangential axes for a given face; mapping to XYZ happens later.
    match n {
        1 => vec![(0.0, 0.0)],
        2 => vec![(-o, o), (o, -o)],
        3 => vec![(-o, o), (0.0, 0.0), (o, -o)],
        4 => vec![(-o, o), (o, o), (-o, -o), (o, -o)],
        5 => vec![(-o, o), (o, o), (0.0, 0.0), (-o, -o), (o, -o)],
        6 => vec![(-o, o), (-o, 0.0), (-o, -o), (o, o), (o, 0.0), (o, -o)],
        _ => vec![],
    }
}
 
fn face_pips(face: (i32, i32, i32), number: u8, o: f64, face_offset: f64) -> Vec<(f64, f64, f64)> {
    let (nx, ny, nz) = face;
    let uvs = pip_uv_for_number(number, o);
 
    // Map (u,v) to XYZ on the chosen face.
    // Convention:
    // - +Z/-Z: u=x, v=y
    // - +Y/-Y: u=x, v=z
    // - +X/-X: u=z, v=y
    uvs.into_iter()
        .map(|(u, v)| {
            if nz != 0 {
                (u, v, face_offset * nz as f64)
            } else if ny != 0 {
                (u, face_offset * ny as f64, v)
            } else {
                (face_offset * nx as f64, v, u)
            }
        })
        .collect()
}
 
fn build_die(base_color: Color) -> Box<dyn Shape> {
    // Build a single die in its own object space:
    // - base cube: [-1,1] on each axis
    // - pips: small spheres slightly pushed out of each face so subtraction always intersects
    // - overall die size/rotation/position is applied later by setting transform on the returned shape.
 
    let mut cube = Cube::new();
    {
        let m = cube.material_mut();
        *m = Material::new();
        m.color = base_color.clone();
        m.ambient = 0.15;
        m.diffuse = 0.75;
        m.specular = 0.35;
        m.shininess = 200.0;
    }
 
    let mut shape: Box<dyn Shape> = Box::new(cube);
 
    let pip_radius = 0.22;
    let pip_offset = 0.55;
    let face_offset = 1.05;
 
    // Opposite faces sum to 7. One common orientation:
    // +Z=1, -Z=6, +Y=2, -Y=5, +X=3, -X=4
    let faces: [((i32, i32, i32), u8); 6] = [
        ((0, 0, 1), 1),
        ((0, 0, -1), 6),
        ((0, 1, 0), 2),
        ((0, -1, 0), 5),
        ((1, 0, 0), 3),
        ((-1, 0, 0), 4),
    ];
 
    for (face, number) in faces {
        for (x, y, z) in face_pips(face, number, pip_offset, face_offset) {
            let mut pip = Sphere::new();
            pip.set_transform(&translation(x, y, z) * &scaling(pip_radius, pip_radius, pip_radius));
            {
                // Slightly darker material for the indent surface.
                let m = pip.material_mut();
                *m = Material::new();
                m.color = Color::new(
                    (base_color.red * 0.55).min(1.0),
                    (base_color.green * 0.55).min(1.0),
                    (base_color.blue * 0.55).min(1.0),
                );
                m.ambient = 0.05;
                m.diffuse = 0.9;
                m.specular = 0.05;
                m.shininess = 50.0;
            }
 
            shape = Box::new(CSG::new(
                CSGOperation::Difference,
                shape,
                Box::new(pip),
            ));
        }
    }
 
    shape
}
 
fn main() {
    let mut world = World::new();
 
    // Floor: soft reflective checkers, similar to the reference.
    let mut floor_pattern = CheckersPattern::new(Color::new(0.92, 0.92, 0.92), Color::new(0.75, 0.75, 0.75));
    floor_pattern.set_transform(scaling(1.25, 1.0, 1.25));
 
    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.ambient = 0.2;
        m.diffuse = 0.75;
        m.specular = 0.05;
        m.reflective = 0.45;
        m.pattern = Some(Box::new(floor_pattern));
    }
    world.add_shape(floor);
 
    // Dice (CSG): same half-extent; bottom two are floor-aligned, green sits on the higher of the two tops.
    let rs_blue = rot_scale_matrix(-0.32, 0.12, 0.0, DIE_HALF);
    let (lo_b, hi_b) = cube_y_extents_after_rot_scale(&rs_blue);
    let ty_blue = -lo_b;
    let tx_blue = -1.15;
    let tz_blue = -0.08;

    let rs_red = rot_scale_matrix(0.48, -0.08, 0.0, DIE_HALF);
    let (lo_r, hi_r) = cube_y_extents_after_rot_scale(&rs_red);
    let ty_red = -lo_r;
    let tx_red = 0.88;
    let tz_red = 0.40;

    let top_y = (ty_blue + hi_b).max(ty_red + hi_r) + STACK_GAP;

    let rs_green = rot_scale_matrix(0.14, 0.0, 0.22, DIE_HALF);
    let (lo_g, _) = cube_y_extents_after_rot_scale(&rs_green);
    let ty_green = top_y - lo_g;
    let tx_green = 0.5 * (tx_blue + tx_red) + 0.02;
    let tz_green = 0.5 * (tz_blue + tz_red) - 0.12;

    let mut die_blue = build_die(Color::new(0.20, 0.25, 0.85));
    die_blue.set_transform(die_transform(tx_blue, ty_blue, tz_blue, &rs_blue));
    world.objects.push(die_blue);
 
    let mut die_red = build_die(Color::new(0.75, 0.25, 0.30));
    die_red.set_transform(die_transform(tx_red, ty_red, tz_red, &rs_red));
    world.objects.push(die_red);
 
    let mut die_green = build_die(Color::new(0.20, 0.70, 0.35));
    die_green.set_transform(die_transform(tx_green, ty_green, tz_green, &rs_green));
    world.objects.push(die_green);
 
    world.lights = vec![
        PointLight::new(Tuple::point(-8.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)),
        // subtle fill so pip cavities aren't completely black
        PointLight::new(Tuple::point(6.0, 4.0, -2.0), Color::new(0.25, 0.25, 0.25)),
    ];
 
    let mut camera = Camera::new(1200, 800, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(-3.8, 3.2, -7.5),
        &Tuple::point(0.0, 0.9, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );
 
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let out = "media/images_ppm/dice_csg.ppm";
    std::fs::write(out, ppm).expect("Failed to write media/images_ppm/dice_csg.ppm");
    println!("Saved to {out}");
}

