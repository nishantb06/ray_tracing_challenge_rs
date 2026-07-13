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
use std::f64::consts::{FRAC_PI_3, TAU};

fn hex_base_vertices(radius: f64) -> [Tuple; 6] {
    std::array::from_fn(|k| {
        let theta = TAU * (k as f64) / 6.0 + FRAC_PI_3 / 3.0; // rotate so a flat edge faces +z (optional)
        Tuple::point(radius * theta.cos(), 0.0, radius * theta.sin())
    })
}

fn add_triangle(world: &mut World, p1: Tuple, p2: Tuple, p3: Tuple, color: Color) {
    let mut tri = Triangle::new(p1, p2, p3);
    {
        let m = tri.material_mut();
        *m = Material::new();
        m.color = color;
        m.ambient = 0.08;
        m.diffuse = 0.85;
        m.specular = 0.25;
        m.shininess = 64.0;
    }
    world.add_shape(tri);
}

fn main() {
    let base_r = 1.0;
    let apex_y = 1.35;
    let apex = Tuple::point(0.0, apex_y, 0.0);
    let center = Tuple::point(0.0, 0.0, 0.0);
    let corners = hex_base_vertices(base_r);

    let mut world = World::new();

    // Sky / backdrop
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

    // 6 lateral faces: apex — corner_i — corner_{i+1}
    let side_color = Color::new(0.85, 0.35, 0.2);
    for i in 0..6 {
        let p0 = corners[i].clone();
        let p1 = corners[(i + 1) % 6].clone();
        add_triangle(&mut world, apex.clone(), p0, p1, side_color.clone());
    }

    // Hex base (fan from center); winding gives outward normal −Y
    let base_color = Color::new(0.25, 0.45, 0.75);
    for i in 0..6 {
        let a = corners[(i + 1) % 6].clone();
        let b = corners[i].clone();
        add_triangle(&mut world, center.clone(), a, b, base_color.clone());
    }

    world.lights = vec![PointLight::new(
        Tuple::point(-5.0, 8.0, -6.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(2.4, 1.1, -3.4),
        &Tuple::point(0.0, 0.45, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let path = "media/images_ppm/hex_cone.ppm";
    std::fs::write(path, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", path);
}