use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::{Color, BLACK, WHITE};
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_y, scaling, translation, view_transform};
use ray_tracing_challenge_rs::triangle::Triangle;
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

/// Vertex list + polygon (pent/hex) indices ported from JSModeler
/// `GenerateTruncatedIcosahedron` (kovacsv/JSModeler).
fn truncated_icosahedron_vertices(radius: f64) -> Vec<Tuple> {
    let d = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let e = 3.0 * d;
    let f = 1.0 + 2.0 * d;
    let g = 2.0 + d;
    let h = 2.0 * d;

    let a = 0.0;
    let b = 1.0;
    let c = 2.0;

    let raw: [(f64, f64, f64); 60] = [
        // 0..3
        (a, b, e),
        (a, b, -e),
        (a, -b, e),
        (a, -b, -e),
        // 4..7
        (b, e, a),
        (b, -e, a),
        (-b, e, a),
        (-b, -e, a),
        // 8..11
        (e, a, b),
        (-e, a, b),
        (e, a, -b),
        (-e, a, -b),
        // 12..19
        (c, f, d),
        (c, f, -d),
        (c, -f, d),
        (-c, f, d),
        (c, -f, -d),
        (-c, f, -d),
        (-c, -f, d),
        (-c, -f, -d),
        // 20..27
        (f, d, c),
        (f, -d, c),
        (-f, d, c),
        (f, d, -c),
        (-f, -d, c),
        (f, -d, -c),
        (-f, d, -c),
        (-f, -d, -c),
        // 28..35
        (d, c, f),
        (-d, c, f),
        (d, c, -f),
        (d, -c, f),
        (-d, c, -f),
        (-d, -c, f),
        (d, -c, -f),
        (-d, -c, -f),
        // 36..43
        (b, g, h),
        (b, g, -h),
        (b, -g, h),
        (-b, g, h),
        (b, -g, -h),
        (-b, g, -h),
        (-b, -g, h),
        (-b, -g, -h),
        // 44..51
        (g, h, b),
        (g, -h, b),
        (-g, h, b),
        (g, h, -b),
        (-g, -h, b),
        (g, -h, -b),
        (-g, h, -b),
        (-g, -h, -b),
        // 52..59
        (h, b, g),
        (-h, b, g),
        (h, b, -g),
        (h, -b, g),
        (-h, b, -g),
        (-h, -b, g),
        (h, -b, -g),
        (-h, -b, -g),
    ];

    raw.into_iter()
        .map(|(x, y, z)| {
            let len = (x * x + y * y + z * z).sqrt();
            Tuple::point(x / len * radius, y / len * radius, z / len * radius)
        })
        .collect()
}

const PENTAGONS: &[[usize; 5]] = &[
    [0, 28, 36, 39, 29],
    [1, 32, 41, 37, 30],
    [2, 33, 42, 38, 31],
    [3, 34, 40, 43, 35],
    [4, 12, 44, 47, 13],
    [5, 16, 49, 45, 14],
    [6, 17, 50, 46, 15],
    [7, 18, 48, 51, 19],
    [8, 20, 52, 55, 21],
    [9, 24, 57, 53, 22],
    [10, 25, 58, 54, 23],
    [11, 26, 56, 59, 27],
];

const HEXAGONS: &[[usize; 6]] = &[
    [0, 2, 31, 55, 52, 28],
    [0, 29, 53, 57, 33, 2],
    [1, 3, 35, 59, 56, 32],
    [1, 30, 54, 58, 34, 3],
    [4, 6, 15, 39, 36, 12],
    [4, 13, 37, 41, 17, 6],
    [5, 7, 19, 43, 40, 16],
    [5, 14, 38, 42, 18, 7],
    [8, 10, 23, 47, 44, 20],
    [8, 21, 45, 49, 25, 10],
    [9, 11, 27, 51, 48, 24],
    [9, 22, 46, 50, 26, 11],
    [12, 36, 28, 52, 20, 44],
    [13, 47, 23, 54, 30, 37],
    [14, 45, 21, 55, 31, 38],
    [15, 46, 22, 53, 29, 39],
    [16, 40, 34, 58, 25, 49],
    [17, 41, 32, 56, 26, 50],
    [18, 42, 33, 57, 24, 48],
    [19, 51, 27, 59, 35, 43],
];

fn add_triangle(group: &mut Group, verts: &[Tuple], i0: usize, i1: usize, i2: usize, material: &Material) {
    let mut t = Triangle::new(verts[i0].clone(), verts[i1].clone(), verts[i2].clone());
    *t.material_mut() = Material {
        color: material.color.clone(),
        ambient: material.ambient,
        diffuse: material.diffuse,
        specular: material.specular,
        shininess: material.shininess,
        pattern: None,
        reflective: material.reflective,
        transparency: material.transparency,
        refractive_index: material.refractive_index,
    };
    group.add_child(Box::new(t));
}

fn add_polygon_fan<const N: usize>(
    group: &mut Group,
    verts: &[Tuple],
    poly: &[usize; N],
    material: &Material,
    flip_winding: bool,
) {
    let v0 = poly[0];
    for i in 1..(N - 1) {
        let v1 = poly[i];
        let v2 = poly[i + 1];
        if flip_winding {
            add_triangle(group, verts, v0, v2, v1, material);
        } else {
            add_triangle(group, verts, v0, v1, v2, material);
        }
    }
}

fn main() {
    // If the ball looks "inside-out" (lighting inverted), set this to true.
    const FLIP_WINDING: bool = false;

    let mut world = World::new();

    // Soft backdrop to make the ball edges readable.
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(0.92, 0.94, 0.97);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    world.add_shape(background);

    let verts = truncated_icosahedron_vertices(1.2);

    let mut pent_mat = Material::new();
    pent_mat.color = BLACK.clone();
    pent_mat.ambient = 0.08;
    pent_mat.diffuse = 0.85;
    pent_mat.specular = 0.25;
    pent_mat.shininess = 60.0;

    let mut hex_mat = Material::new();
    hex_mat.color = WHITE.clone();
    hex_mat.ambient = 0.10;
    hex_mat.diffuse = 0.85;
    hex_mat.specular = 0.35;
    hex_mat.shininess = 90.0;

    let mut ball = Group::new();
    ball.set_transform(
        &(&rotation_y(0.55) * &translation(0.0, -0.12, 0.0)) * &scaling(1.05, 1.05, 1.05),
    );

    for p in PENTAGONS {
        add_polygon_fan(&mut ball, &verts, p, &pent_mat, FLIP_WINDING);
    }
    for h in HEXAGONS {
        add_polygon_fan(&mut ball, &verts, h, &hex_mat, FLIP_WINDING);
    }

    world.add_shape(ball);

    world.lights = vec![
        PointLight::new(Tuple::point(-6.0, 10.0, -5.0), Color::new(0.9, 0.9, 0.9)),
        PointLight::new(Tuple::point(7.0, 8.0, -2.0), Color::new(0.35, 0.35, 0.35)),
    ];

    let mut camera = Camera::new(900, 700, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(3.3, 2.4, -4.8),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let path = "media/images_ppm/football.ppm";
    std::fs::write(path, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", path);
}
