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

const HEX_COLOR: Color = Color {
    red: 0.2,
    green: 0.8,
    blue: 1.0,
};

fn hexagon_corner() -> &'static mut Sphere {
    let corner: &'static mut Sphere = Box::leak(Box::new(Sphere::new()));
    corner.set_transform(&translation(0.0, 0.0, -1.0) * &scaling(0.25, 0.25, 0.25));
    corner.material_mut().color = HEX_COLOR;
    
    corner
}

fn hexagon_edge() -> &'static mut Cylinder {
    let edge: &'static mut Cylinder = Box::leak(Box::new(Cylinder::new()));
    edge.minimum = 0.0;
    edge.maximum = 1.0;
    edge.set_transform(
        &(&(&translation(0.0, 0.0, -1.0) * &rotation_y(-PI / 6.0)) * &rotation_z(-FRAC_PI_2))
            * &scaling(0.25, 1.0, 0.25),
    );
    edge.material_mut().color = HEX_COLOR;
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

fn hexagon() -> Group<'static> {
    let mut hex = Group::new();

    for n in 0..6 {
        // Create side, leak it so parent group can store a 'static reference
        let side: &'static mut Group<'static> = Box::leak(Box::new(hexagon_side()));
        side.set_transform(rotation_y(n as f64 * PI / 3.0));

        side.shape_data_mut().parent = Some(hex.id());
        hex.add_child(side);
    }

    hex
}

fn main() {
    let mut world = World::new();
    world.lights = vec![
        // Key lights
        PointLight::new(Tuple::point(-10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(-10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
    
        // Mid-height rim/fill lights
        PointLight::new(Tuple::point(-14.0, 6.0, 0.0), Color::new(0.20, 0.20, 0.20)),
        PointLight::new(Tuple::point(14.0, 6.0, 0.0), Color::new(0.20, 0.20, 0.20)),
        PointLight::new(Tuple::point(0.0, 6.0, -14.0), Color::new(0.20, 0.20, 0.20)),
        PointLight::new(Tuple::point(0.0, 6.0, 14.0), Color::new(0.20, 0.20, 0.20)),
    
        // Low fill lights to lift shadows
        PointLight::new(Tuple::point(-6.0, 2.0, -6.0), Color::new(0.12, 0.12, 0.12)),
        PointLight::new(Tuple::point(6.0, 2.0, -6.0), Color::new(0.12, 0.12, 0.12)),
        PointLight::new(Tuple::point(-6.0, 2.0, 6.0), Color::new(0.12, 0.12, 0.12)),
        PointLight::new(Tuple::point(6.0, 2.0, 6.0), Color::new(0.12, 0.12, 0.12)),
    ];

    world.add_shape(hexagon());

    let mut camera = Camera::new(800, 500, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 2.5, -6.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/group_hexagon.ppm", ppm)
        .expect("Failed to write media/images_ppm/group_hexagon.ppm");
    println!("Saved to media/images_ppm/group_hexagon.ppm");
}