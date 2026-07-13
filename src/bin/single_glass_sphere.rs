use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Checkered floor
    let mut floor_pattern = CheckersPattern::new(
        Color::new(1.0, 1.0, 1.0),
        Color::new(0.0, 0.0, 0.0),
    );
    floor_pattern.set_transform(scaling(1.0, 1.0, 1.0));

    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.ambient = 0.2;
        m.diffuse = 0.8;
        m.specular = 0.0;
        m.reflective = 0.1;
        m.pattern = Some(Box::new(floor_pattern));
    }

    // Simple vertical back wall to give context
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

    // Single glass sphere
    let mut glass_sphere = Sphere::glass_sphere();
    glass_sphere.set_transform(
        &translation(0.0, 1.0, 0.5) * &scaling(1.5, 1.5, 1.5),
    );
    {
        let m = glass_sphere.material_mut();
        m.color = Color::new(0.7, 0.9, 0.8);
        m.ambient = 0.05;
        m.diffuse = 0.2;
        m.specular = 0.9;
        m.shininess = 300.0;
        m.transparency = 0.9;
        m.reflective = 0.4;
        m.refractive_index = 1.5;
    }

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(glass_sphere);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(1920, 1080, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 2.0, -6.0),
        &Tuple::point(0.0, 1.0, 1.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/single_glass_sphere.ppm", ppm)
        .expect("Failed to write single_glass_sphere.ppm");
    println!("Saved to media/images_ppm/single_glass_sphere.ppm");
}

