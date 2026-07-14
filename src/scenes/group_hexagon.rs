use crate::camera::Camera;
use crate::canvas::Color;
use crate::cylinder::Cylinder;
use crate::group::Group;
use crate::light::PointLight;
use crate::shape::Shape;
use crate::sphere::Sphere;
use crate::transformation::{rotation_y, rotation_z, scaling, translation, view_transform};
use crate::tuple::Tuple;
use crate::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, PI};

const HEX_COLOR: Color = Color {
    red: 0.2,
    green: 0.8,
    blue: 1.0,
};

fn hexagon_corner() -> Sphere {
    let mut corner = Sphere::new();
    corner.set_transform(&translation(0.0, 0.0, -1.0) * &scaling(0.25, 0.25, 0.25));
    corner.material_mut().color = HEX_COLOR;
    corner
}

fn hexagon_edge() -> Cylinder {
    let mut edge = Cylinder::new();
    edge.minimum = 0.0;
    edge.maximum = 1.0;
    edge.set_transform(
        &(&(&translation(0.0, 0.0, -1.0) * &rotation_y(-PI / 6.0)) * &rotation_z(-FRAC_PI_2))
            * &scaling(0.25, 1.0, 0.25),
    );
    edge.material_mut().color = HEX_COLOR;
    edge
}

fn hexagon_side() -> Group {
    let mut side = Group::new();
    side.add_child(Box::new(hexagon_corner()));
    side.add_child(Box::new(hexagon_edge()));
    side
}

fn hexagon() -> Group {
    let mut hex = Group::new();
    for n in 0..6 {
        let mut side = hexagon_side();
        side.set_transform(rotation_y(n as f64 * PI / 3.0));
        hex.add_child(Box::new(side));
    }
    hex
}

pub fn build(width: usize, height: usize) -> (Camera, World) {
    let mut world = World::new();
    world.lights = vec![
        PointLight::new(Tuple::point(-10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, -10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(-10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
        PointLight::new(Tuple::point(10.0, 12.0, 10.0), Color::new(0.35, 0.35, 0.35)),
    ];
    world.add_shape(hexagon());

    let mut camera = Camera::new(width, height, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(1.0, 1.25, -3.0),
        &Tuple::point(0.0, 0.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    (camera, world)
}
