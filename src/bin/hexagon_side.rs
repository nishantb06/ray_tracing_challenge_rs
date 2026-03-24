use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, PI};

fn hexagon_corner() -> &'static mut Sphere {
    let corner: &'static mut Sphere = Box::leak(Box::new(Sphere::new()));
    corner.set_transform(
        &translation(0.0, 0.0, -1.0) * &scaling(0.25, 0.25, 0.25),
    );
    corner.material_mut().color = Color::new(0.9, 0.2, 0.3);
    corner
}

fn hexagon_edge() -> &'static mut Cylinder {
    let edge: &'static mut Cylinder = Box::leak(Box::new(Cylinder::new()));
    edge.minimum = 0.0;
    edge.maximum = 1.0;
    edge.set_transform(
        &(&(&translation(0.0, 0.0, -1.0) * &rotation_y(-PI / 6.0))
            * &rotation_z(-FRAC_PI_2))
            * &scaling(0.25, 1.0, 0.25),
    );
    edge.material_mut().color = Color::new(0.9, 0.2, 0.3);
    edge
}

fn hexagon_side() -> Group<'static> {
    let mut side = Group::new();

    let corner = hexagon_corner();
    corner.shape_data_mut().parent = Some(side.id());
    side.add_child(corner);

    let edge = hexagon_edge();
    edge.shape_data_mut().parent = Some(side.id());
    side.add_child(edge);

    side
}

fn main() {
    let mut world = World::new();
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    // Only one side for now (as requested)
    let mut side = hexagon_side();
    side.set_transform(rotation_y(PI / 4.0)); // 45 degrees
    world.add_shape(side);

    let mut camera = Camera::new(600, 300, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 2.0, -5.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/hexagon_side.ppm", ppm)
        .expect("Failed to write media/images_ppm/hexagon_side.ppm");
    println!("Saved to media/images_ppm/hexagon_side.ppm");
}