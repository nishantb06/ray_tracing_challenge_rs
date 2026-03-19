use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cone::Cone;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::transformation::{scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    // Floor plane
    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.9, 0.9);
        m.specular = 0.0;
        m.reflective = 0.1;
    }

    // Cylinder on the left: capped, height 2, centered at y=1 so it sits on the floor
    let mut cylinder = Cylinder::new();
    cylinder.minimum = -1.0;
    cylinder.maximum = 1.0;
    cylinder.closed = true;
    cylinder.set_transform(translation(-1.8, 1.0, 0.0));
    {
        let m = cylinder.material_mut();
        *m = Material::new();
        m.color = Color::new(0.2, 0.5, 0.9);
        m.diffuse = 0.8;
        m.specular = 0.4;
        m.shininess = 50.0;
    }

    // Cone on the right: capped, minimum=-1, maximum=0 so it points upward with tip at top.
    // Scaled by 1.5 to make it visible, then translated so its base (y=-1 * scale = -1.5) sits
    // on the floor at y=0, meaning we translate up by 1.5.
    let mut cone = Cone::new();
    cone.minimum = -1.0;
    cone.maximum = 0.0;
    cone.closed = true;
    cone.set_transform(&translation(1.8, 1.5, 0.0) * &scaling(1.5, 1.5, 1.5));
    {
        let m = cone.material_mut();
        *m = Material::new();
        m.color = Color::new(0.9, 0.3, 0.2);
        m.diffuse = 0.8;
        m.specular = 0.4;
        m.shininess = 50.0;
    }

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(cylinder);
    world.add_shape(cone);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(800, 600, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(0.0, 3.5, -6.0),
        &Tuple::point(0.0, 1.0, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/cone_and_cylinder.ppm", ppm)
        .expect("Failed to write media/images_ppm/cone_and_cylinder.ppm");
    println!("Saved to media/images_ppm/cone_and_cylinder.ppm");
}
