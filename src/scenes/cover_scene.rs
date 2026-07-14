use crate::camera::Camera;
use crate::canvas::Color;
use crate::cube::Cube;
use crate::light::PointLight;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::plane::Plane;
use crate::shape::Shape;
use crate::sphere::Sphere;
use crate::transformation::{rotation_x, scaling, translation, view_transform};
use crate::tuple::Tuple;
use crate::world::World;

fn white_material() -> Material {
    Material {
        color: Color::new(1.0, 1.0, 1.0),
        diffuse: 0.7,
        ambient: 0.1,
        specular: 0.0,
        reflective: 0.1,
        ..Material::new()
    }
}

fn blue_material() -> Material {
    Material {
        color: Color::new(0.537, 0.831, 0.914),
        ..white_material()
    }
}

fn red_material() -> Material {
    Material {
        color: Color::new(0.941, 0.322, 0.388),
        ..white_material()
    }
}

fn purple_material() -> Material {
    Material {
        color: Color::new(0.373, 0.404, 0.550),
        ..white_material()
    }
}

fn standard_transform() -> Matrix {
    &scaling(0.5, 0.5, 0.5) * &translation(1.0, -1.0, 1.0)
}

fn large_object() -> Matrix {
    &scaling(3.5, 3.5, 3.5) * &standard_transform()
}

fn medium_object() -> Matrix {
    &scaling(3.0, 3.0, 3.0) * &standard_transform()
}

fn small_object() -> Matrix {
    &scaling(2.0, 2.0, 2.0) * &standard_transform()
}

fn make_cube(mat: Material, transform: Matrix) -> Cube {
    let mut c = Cube::new();
    c.set_transform(transform);
    *c.material_mut() = mat;
    c
}

pub fn build(width: usize, height: usize) -> (Camera, World) {
    let mut world = World::new();

    world.lights = vec![
        PointLight::new(Tuple::point(50.0, 100.0, -50.0), Color::new(1.0, 1.0, 1.0)),
        PointLight::new(Tuple::point(-400.0, 50.0, -10.0), Color::new(0.2, 0.2, 0.2)),
    ];

    let mut backdrop = Plane::new();
    {
        let m = backdrop.material_mut();
        m.color = Color::new(1.0, 1.0, 1.0);
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
    }
    backdrop.set_transform(
        &translation(0.0, 0.0, 500.0) * &rotation_x(std::f64::consts::FRAC_PI_2),
    );
    world.add_shape(backdrop);

    let mut glass = Sphere::new();
    {
        let m = glass.material_mut();
        m.color = Color::new(0.373, 0.404, 0.550);
        m.diffuse = 0.2;
        m.ambient = 0.0;
        m.specular = 1.0;
        m.shininess = 200.0;
        m.reflective = 0.7;
        m.transparency = 0.7;
        m.refractive_index = 1.5;
    }
    glass.set_transform(large_object());
    world.add_shape(glass);

    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 0.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        blue_material(),
        &translation(8.5, 1.5, -0.5) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(0.0, 0.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(4.0, 0.0, 4.0) * &small_object(),
    ));
    world.add_shape(make_cube(
        purple_material(),
        &translation(7.5, 0.5, 4.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.25, 0.25, 8.0) * &medium_object(),
    ));
    world.add_shape(make_cube(
        blue_material(),
        &translation(4.0, 1.0, 7.5) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(10.0, 2.0, 7.5) * &medium_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(8.0, 2.0, 12.0) * &small_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(20.0, 1.0, 9.0) * &small_object(),
    ));

    world.add_shape(make_cube(
        blue_material(),
        &translation(-0.5, -5.0, 0.25) * &large_object(),
    ));
    world.add_shape(make_cube(
        red_material(),
        &translation(4.0, -4.0, 0.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(8.5, -4.0, 0.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -4.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        purple_material(),
        &translation(-0.5, -4.5, 8.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(0.0, -8.0, 4.0) * &large_object(),
    ));
    world.add_shape(make_cube(
        white_material(),
        &translation(-0.5, -8.5, 8.0) * &large_object(),
    ));

    let mut camera = Camera::new(width, height, 0.785);
    camera.set_transform(view_transform(
        &Tuple::point(-6.0, 6.0, -10.0),
        &Tuple::point(6.0, 0.0, 6.0),
        &Tuple::vector(-0.45, 1.0, 0.0),
    ));

    (camera, world)
}
