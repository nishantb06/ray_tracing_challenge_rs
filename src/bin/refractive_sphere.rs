use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_3};

fn main() {
    // --- Floor: black & white checkered plane, slightly reflective ---
    let mut floor_pattern = CheckersPattern::new(
        Color::new(1.0, 1.0, 1.0), // white
        Color::new(0.0, 0.0, 0.0), // black
    );
    // Make the checks reasonably sized
    floor_pattern.set_transform(scaling(1.0, 1.0, 1.0));

    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.ambient = 0.2;
        m.diffuse = 0.8;
        m.specular = 0.0;
        m.reflective = 0.2;
        m.pattern = Some(Box::new(floor_pattern));
    }

    // --- Walls: two large planes forming a corner, both using a subtle gray material ---
    let mut back_wall = Plane::new();
    back_wall.set_transform(
        &translation(0.0, 0.0, 8.0) * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    {
        let m = back_wall.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.9, 0.9);
        m.ambient = 0.1;
        m.diffuse = 0.7;
        m.specular = 0.0;
    }

    let mut side_wall = Plane::new();
    side_wall.set_transform(
        &(&translation(-6.0, 0.0, 0.0) * &rotation_z(-std::f64::consts::FRAC_PI_2))
            * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    {
        let m = side_wall.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.9, 0.9);
        m.ambient = 0.1;
        m.diffuse = 0.7;
        m.specular = 0.0;
    }

    // --- Transparent glass sphere in front ---
    let mut glass_sphere = Sphere::glass_sphere();
    glass_sphere.set_transform(
        &translation(0.0, 1.0, 0.0) * &scaling(1.5, 1.5, 1.5),
    );
    {
        let m = glass_sphere.material_mut();
        // Strong glassy look: mostly clear, slightly tinted
        m.color = Color::new(0.6, 0.8, 0.7);
        m.ambient = 0.05;
        m.diffuse = 0.2;
        m.specular = 0.9;
        m.shininess = 300.0;
        m.transparency = 0.8;
        m.reflective = 0.3;
        m.refractive_index = 1.5;
    }

    // --- Solid sphere behind the glass (so refraction is visible) ---
    let mut solid_sphere = Sphere::new();
    solid_sphere.set_transform(
        &translation(0.0, 1.0, 3.0) * &scaling(1.5, 1.5, 1.5),
    );
    {
        let m = solid_sphere.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.1, 0.1); // rich red
        m.ambient = 0.1;
        m.diffuse = 0.7;
        m.specular = 0.3;
        m.shininess = 150.0;
    }

    // --- An additional small solid sphere off to the side for visual interest ---
    let mut side_sphere = Sphere::new();
    side_sphere.set_transform(
        &translation(3.0, 0.5, 2.5) * &scaling(0.5, 0.5, 0.5),
    );
    {
        let m = side_sphere.material_mut();
           *m = Material::new();
        m.color = Color::new(0.2, 0.4, 1.0); // blue-ish
        m.ambient = 0.1;
        m.diffuse = 0.7;
        m.specular = 0.4;
        m.shininess = 100.0;
        m.reflective = 0.1;

    }

    // --- World setup ---
    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(side_wall);
    world.add_shape(solid_sphere);
    world.add_shape(side_sphere);
    world.add_shape(glass_sphere);

    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // --- Camera: capture floor, both walls, and spheres ---
    let mut camera = Camera::new(1920, 1080, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 2.5, -10.0),
        &Tuple::point(0.0, 1.0, 2.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/refractive_sphere.ppm", ppm)
        .expect("Failed to write refractive_sphere.ppm");
    println!("Saved to media/images_ppm/refractive_sphere.ppm");
}

