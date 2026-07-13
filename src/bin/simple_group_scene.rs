use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::view_transform;
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // 1) One sphere (leaked so Group can hold a 'static reference)
    let mut sphere: Sphere = Sphere::new();
    sphere.material_mut().color = Color::new(0.2, 0.8, 1.0);
    sphere.material_mut().diffuse = 0.7;
    sphere.material_mut().specular = 0.3;

    // 2) Put sphere into a group
    let mut group = Group::new();
    // Optional parent linkage (useful when you later rely on parent-chain normal helpers)
    sphere.shape_data_mut().parent = Some(group.id());
    group.add_child(Box::new(sphere));

    // 3) Build world
    let mut world = World::new();
    world.add_shape(group);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // 4) Camera
    let mut camera = Camera::new(400, 200, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 1.5, -5.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    // 5) Render
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/simple_group_scene.ppm", ppm)
        .expect("Failed to write media/images_ppm/simple_group_scene.ppm");
    println!("Saved to media/images_ppm/simple_group_scene.ppm");
}