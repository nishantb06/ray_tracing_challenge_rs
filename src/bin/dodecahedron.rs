use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{scaling, view_transform};
use ray_tracing_challenge_rs::triangle::Triangle;
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

// Vertices + triangle indices: Three.js DodecahedronGeometry (regular dodecahedron).
fn dodecahedron_vertices(radius: f64) -> Vec<Tuple> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let r = 1.0 / phi;
    let t = phi;

    let raw = [
        (-1.0, -1.0, -1.0),
        (-1.0, -1.0, 1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, 1.0, 1.0),
        (1.0, -1.0, -1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, -1.0),
        (1.0, 1.0, 1.0),
        (0.0, -r, -t),
        (0.0, -r, t),
        (0.0, r, -t),
        (0.0, r, t),
        (-r, -t, 0.0),
        (-r, t, 0.0),
        (r, -t, 0.0),
        (r, t, 0.0),
        (-t, 0.0, -r),
        (t, 0.0, -r),
        (-t, 0.0, r),
        (t, 0.0, r),
    ];

    raw
        .into_iter()
        .map(|(x, y, z)| {
            let len = (x * x + y * y + z * z).sqrt();
            Tuple::point(x / len * radius, y / len * radius, z / len * radius)
        })
        .collect()
}

/// 12 pentagons × 3 triangles each (triangle list).
const DODECA_INDEX: &[usize] = &[
    3, 11, 7, 3, 7, 15, 3, 15, 13, 7, 19, 17, 7, 17, 6, 7, 6, 15, 17, 4, 8, 17, 8, 10, 17, 10, 6,
    8, 0, 16, 8, 16, 2, 8, 2, 10, 0, 12, 1, 0, 1, 18, 0, 18, 16, 6, 10, 2, 6, 2, 13, 6, 13, 15,
    2, 16, 18, 2, 18, 3, 2, 3, 13, 18, 1, 9, 18, 9, 11, 18, 11, 3, 4, 14, 12, 4, 12, 0, 4, 0, 8,
    11, 9, 5, 11, 5, 19, 11, 19, 7, 19, 5, 14, 19, 14, 4, 19, 4, 17, 1, 12, 14, 1, 14, 5, 1, 5, 9,
];

fn add_triangle(world: &mut World, p1: Tuple, p2: Tuple, p3: Tuple, color: Color) {
    let mut tri = Triangle::new(p1, p2, p3);
    {
        let m = tri.material_mut();
        *m = Material::new();
        m.color = color;
        m.ambient = 0.09;
        m.diffuse = 0.82;
        m.specular = 0.35;
        m.shininess = 80.0;
    }
    world.add_shape(tri);
}

fn main() {
    let mut world = World::new();

    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.92, 0.96);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    world.add_shape(background);

    let verts = dodecahedron_vertices(1.15);
    let gold = Color::new(0.78, 0.62, 0.28);
    let gold_dark = Color::new(0.58, 0.44, 0.22);

    for (i, tri) in DODECA_INDEX.chunks_exact(3).enumerate() {
        let c = if (i / 3) % 2 == 0 {
            gold.clone()
        } else {
            gold_dark.clone()
        };
        add_triangle(
            &mut world,
            verts[tri[0]].clone(),
            verts[tri[1]].clone(),
            verts[tri[2]].clone(),
            c,
        );
    }

    world.lights = vec![PointLight::new(
        Tuple::point(-6.0, 10.0, -5.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(3.0, 2.2, -4.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let path = "media/images_ppm/dodecahedron.ppm";
    std::fs::write(path, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", path);
}