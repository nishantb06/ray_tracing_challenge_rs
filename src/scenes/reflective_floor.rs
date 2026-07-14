use crate::camera::Camera;
use crate::canvas::Color;
use crate::light::PointLight;
use crate::material::Material;
use crate::pattern::{CheckersPattern, Pattern};
use crate::plane::Plane;
use crate::shape::Shape;
use crate::sphere::Sphere;
use crate::transformation::{rotation_x, rotation_z, scaling, translation, view_transform};
use crate::tuple::Tuple;
use crate::world::World;
use std::f64::consts::FRAC_PI_3;

pub fn build(width: usize, height: usize) -> (Camera, World) {
    let mut floor_pattern =
        CheckersPattern::new(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0));
    floor_pattern.set_transform(scaling(1.5, 1.0, 1.5));

    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.ambient = 0.2;
        m.diffuse = 0.7;
        m.specular = 0.1;
        m.pattern = Some(Box::new(floor_pattern));
        m.reflective = 0.6;
    }

    let mut ball = Sphere::new();
    ball.set_transform(translation(0.0, 1.0, 0.0));
    {
        let m = ball.material_mut();
        *m = Material::new();
        m.color = Color::new(1.0, 0.1, 0.1);
        m.ambient = 0.1;
        m.diffuse = 0.7;
        m.specular = 0.3;
        m.shininess = 100.0;
    }

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
        &(&translation(6.0, 0.0, 0.0) * &rotation_z(std::f64::consts::FRAC_PI_2))
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

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(back_wall);
    world.add_shape(side_wall);
    world.add_shape(ball);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(width, height, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 2.0, -9.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    (camera, world)
}
