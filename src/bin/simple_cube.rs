use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::transformation::{scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use ray_tracing_challenge_rs::sphere::Sphere;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // White background: a huge matte sphere surrounding the scene.
    // This ensures rays that miss the cube still return white.
    let mut background = Sphere::new();
    background.set_transform(scaling(1000.0, 1000.0, 1000.0));
    {
        let m = background.material_mut();
        *m = Material::new();
        m.color = Color::new(1.0, 1.0, 1.0);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
        m.reflective = 0.0;
        m.refractive_index = 1.2;
    }

    // Opaque, non-reflective grey cube, floating in the air.
    let mut cube = Cube::new();
    cube.set_transform(translation(0.0, 0.75, 0.0));
    {
        let m = cube.material_mut();
        *m = Material::new();
        m.color = Color::new(0.6, 0.6, 0.6);
        m.ambient = 0.1;
        m.diffuse = 0.9;
        m.specular = 0.0;
        m.reflective = 0.0;
    }

    let mut world = World::new();
    world.add_shape(background);
    world.add_shape(cube);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // Camera is angled so it isn't face-on with any cube face; you should see a corner.
    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(-3.0, 2.0, -4.0),
        &Tuple::point(0.0, 0.75, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/simple_cube.ppm", ppm)
        .expect("Failed to write media/images_ppm/simple_cube.ppm");
    println!("Saved to media/images_ppm/simple_cube.ppm");
}

