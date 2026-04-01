use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::csg::{CSG, CSGOperation};
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
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
 
    // Dice (CSG): build in local space then transform each instance.
    let mut die_blue = build_die(Color::new(0.20, 0.25, 0.85));
    die_blue.set_transform(&(&(&translation(-1.2, 0.65, 0.0) * &rotation_y(-0.35)) * &rotation_x(0.20)) * &scaling(0.65, 0.65, 0.65));
    world.objects.push(die_blue);
 
    let mut die_red = build_die(Color::new(0.75, 0.25, 0.30));
    die_red.set_transform(&(&(&translation(0.55, 0.65, 0.35) * &rotation_y(0.55)) * &rotation_x(-0.10)) * &scaling(0.65, 0.65, 0.65));
    world.objects.push(die_red);
 
    let mut die_green = build_die(Color::new(0.20, 0.70, 0.35));
    die_green.set_transform(&(&(&translation(0.10, 1.55, -0.25) * &rotation_y(0.15)) * &rotation_z(0.25)) * &scaling(0.65, 0.65, 0.65));
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

