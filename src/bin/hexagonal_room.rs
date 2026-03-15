// Hexagonal room: floor + 6 walls (planes rotated and translated). Camera from above looking down.
use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_2;

fn main() {
    let hex_radius = 4.0;

    // Floor
    let mut floor = Plane::new();
    floor.data.material.color = Color::new(0.9, 0.88, 0.85);
    floor.data.material.specular = 0.0;

    // Six walls: each is a plane rotated to vertical (rotation_x(π/2)), then rotated around y
    // so the normal points outward at angle π/3 * k, then translated to the hexagon edge.
    let wall_colors = [
        Color::new(0.95, 0.9, 0.9),
        Color::new(0.9, 0.92, 0.95),
        Color::new(0.92, 0.95, 0.9),
        Color::new(0.95, 0.92, 0.9),
        Color::new(0.9, 0.9, 0.95),
        Color::new(0.92, 0.9, 0.92),
    ];

    let mut world = World::new();
    world.add_shape(floor);

    for k in 0..6 {
        let angle = (k as f64) * std::f64::consts::FRAC_PI_3; // 0, 60°, 120°, ...
        let nx = angle.sin();
        let nz = angle.cos();
        let mut wall = Plane::new();
        wall.set_transform(
            &(&translation(nx * hex_radius, 0.0, nz * hex_radius) * &rotation_y(angle))
                * &rotation_x(FRAC_PI_2),
        );
        wall.data.material.color = wall_colors[k].clone();
        wall.data.material.specular = 0.0;
        world.add_shape(wall);
    }

    // A few spheres inside the room so the geometry is visible from above
    let mut center = Sphere::new();
    center.set_transform(translation(0.0, 0.4, 0.0));
    center.data.material = Material::new();
    center.data.material.color = Color::new(0.2, 0.6, 1.0);
    center.data.material.diffuse = 0.7;
    center.data.material.specular = 0.3;
    world.add_shape(center);

    let mut off1 = Sphere::new();
    off1.set_transform(translation(1.5, 0.3, 1.2));
    off1.data.material = Material::new();
    off1.data.material.color = Color::new(1.0, 0.5, 0.2);
    off1.data.material.diffuse = 0.7;
    off1.data.material.specular = 0.3;
    world.add_shape(off1);

    let mut off2 = Sphere::new();
    off2.set_transform(translation(-1.2, 0.25, -1.0));
    off2.data.material = Material::new();
    off2.data.material.color = Color::new(0.3, 0.9, 0.4);
    off2.data.material.diffuse = 0.7;
    off2.data.material.specular = 0.3;
    world.add_shape(off2);

    world.lights = vec![PointLight::new(
        Tuple::point(0.0, 3.0, 0.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // Camera from above, looking down at the center
    let mut camera = Camera::new(800, 800, 0.8);
    camera.transform = view_transform(
        &Tuple::point(0.0, 10.0, 0.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 0.0, -1.0), // "up" in image is -z so hexagon orientation looks right
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/hexagonal_room.ppm", ppm)
        .expect("Failed to write hexagonal_room.ppm");
    println!("Saved to media/images_ppm/hexagonal_room.ppm");
}
