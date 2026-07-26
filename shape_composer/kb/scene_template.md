# Scene template (copy the imports EXACTLY — there is NO `prelude` module and NO `ray_tracer` crate)

The library crate is `ray_tracing_challenge_rs` and every type lives in its own
module. The following program compiles as-is; start from it and change the
shapes, transforms, and camera.

```rust
use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{rotation_x, rotation_z, scaling, translation, view_transform};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::FRAC_PI_3;

fn main() {
    let mut floor = Plane::new();
    floor.data.material.color = Color::new(0.9, 0.9, 0.9);
    floor.data.material.specular = 0.0;

    // A cube spans -1..1 per axis: scaling(0.5, 0.75, 0.25) makes a 1.0 x 1.5 x 0.5 box.
    let mut torso = Cube::new();
    torso.set_transform(&translation(0.0, 1.5, 0.0) * &scaling(0.5, 0.75, 0.25));
    torso.data.material.color = Color::new(0.2, 0.4, 0.9);

    let mut head = Sphere::new();
    head.set_transform(&translation(0.0, 2.65, 0.0) * &scaling(0.35, 0.35, 0.35));
    head.data.material.color = Color::new(0.9, 0.7, 0.5);

    // Cylinders are infinite until you set minimum/maximum (public fields, along Y).
    let mut leg = Cylinder::new();
    leg.minimum = 0.0;
    leg.maximum = 1.0;
    leg.closed = true;
    // rotation applies AFTER scaling, translation LAST (rightmost runs first).
    leg.set_transform(&translation(-0.25, 0.0, 0.0) * &scaling(0.12, 0.75, 0.12));
    leg.data.material.color = Color::new(0.3, 0.3, 0.3);

    let mut arm = Cylinder::new();
    arm.minimum = 0.0;
    arm.maximum = 1.0;
    arm.closed = true;
    arm.set_transform(&(&translation(0.55, 2.2, 0.0) * &rotation_z(2.8)) * &scaling(0.09, 0.9, 0.09));
    arm.data.material.color = Color::new(0.9, 0.7, 0.5);
    let _ = rotation_x(0.0); // rotation_x/rotation_y available the same way

    let mut world = World::new();
    world.add_shape(floor);
    world.add_shape(torso);
    world.add_shape(head);
    world.add_shape(leg);
    world.add_shape(arm);
    world.lights = vec![PointLight::new(
        Tuple::point(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    )];

    let mut camera = Camera::new(400, 400, FRAC_PI_3);
    camera.set_transform(view_transform(
        &Tuple::point(0.0, 2.0, -7.0),
        &Tuple::point(0.0, 1.5, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    ));

    let canvas = camera.render(&world);
    std::fs::write(
        "media/images_ppm/shape_composer_scene.ppm",
        canvas.canvas_to_ppm(),
    )
    .expect("write ppm");
    println!("Saved to media/images_ppm/shape_composer_scene.ppm");
}
```

Rules the template demonstrates:
- Matrix composition uses references: `&a * &b`; chain three with `&(&a * &b) * &c`.
- Mutate materials through `shape.data.material.<field>` (color, diffuse, specular, ambient, reflective).
- `world.lights` is a `Vec<PointLight>`; `world.add_shape(shape)` takes ownership.
- The final println MUST stay `Saved to media/images_ppm/shape_composer_scene.ppm`.
