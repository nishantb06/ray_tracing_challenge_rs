//! CSG tree from the diagram:
//!   Final = (Sphere ∩ Cube) − ( (Cone ∪ Cylinder) ∪ SmallSphere )
//!
//! Run: `cargo run --bin csg_tree_demo` → `media/images_ppm/csg_tree_demo.ppm`

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cone::Cone;
use ray_tracing_challenge_rs::csg::{CSG, CSGOperation};
use ray_tracing_challenge_rs::cube::Cube;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::material::Material;
use ray_tracing_challenge_rs::pattern::{CheckersPattern, Pattern};
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};

fn main() {
    let mut world = World::new();

    // --- Floor
    let mut floor_pattern = CheckersPattern::new(
        Color::new(0.88, 0.88, 0.88),
        Color::new(0.68, 0.68, 0.68),
    );
    floor_pattern.set_transform(scaling(1.1, 1.0, 1.1));
    let mut floor = Plane::new();
    {
        let m = floor.material_mut();
        *m = Material::new();
        m.ambient = 0.15;
        m.diffuse = 0.8;
        m.specular = 0.1;
        m.reflective = 0.2;
        m.pattern = Some(Box::new(floor_pattern));
    }
    world.add_shape(floor);

    // --- Left branch: Sphere ∩ Cube  (rounded-cube look when sphere slightly exceeds inscribed sphere)
    let mut cube = Cube::new();
    {
        let m = cube.material_mut();
        m.color = Color::new(0.72, 0.58, 0.42);
        m.ambient = 0.1;
        m.diffuse = 0.75;
        m.specular = 0.35;
        m.shininess = 50.0;
    }

    let mut sphere = Sphere::new();
    sphere.set_transform(scaling(1.22, 1.22, 1.22));
    {
        let m = sphere.material_mut();
        // Match cube so the intersection reads as one surface
        m.color = Color::new(0.72, 0.58, 0.42);
        m.ambient = 0.1;
        m.diffuse = 0.75;
        m.specular = 0.35;
        m.shininess = 50.0;
    }

    let intersection_shape = CSG::new(
        CSGOperation::Intersection,
        Box::new(cube),
        Box::new(sphere),
    );

    // --- Right branch: Union( Union(Cone, Cylinder), SmallSphere )
    let mut cone = Cone::new();
    cone.minimum = 0.0;
    cone.maximum = 1.0;
    cone.closed = true;
    cone.set_transform(
        &(&translation(0.55, -0.35, 0.0) * &rotation_y(-0.35)) * &scaling(0.42, 0.65, 0.42),
    );
    {
        let m = cone.material_mut();
        m.color = Color::new(0.35, 0.55, 0.85);
        m.ambient = 0.08;
        m.diffuse = 0.8;
        m.specular = 0.25;
        m.shininess = 40.0;
    }

    let mut cylinder = Cylinder::new();
    cylinder.minimum = -1.0;
    cylinder.maximum = 1.0;
    cylinder.closed = true;
    // Cylinder on Y; rotate so it cuts along +X through the cube
    cylinder.set_transform(
        &(&translation(-0.55, 0.05, 0.15) * &rotation_z(FRAC_PI_2)) * &scaling(0.22, 0.55, 0.22),
    );
    {
        let m = cylinder.material_mut();
        m.color = Color::new(0.85, 0.4, 0.35);
        m.ambient = 0.08;
        m.diffuse = 0.8;
        m.specular = 0.2;
        m.shininess = 60.0;
    }

    let union_lower = CSG::new(
        CSGOperation::Union,
        Box::new(cone),
        Box::new(cylinder),
    );

    let mut cluster_sphere = Sphere::new();
    cluster_sphere.set_transform(
        &translation(0.15, 0.25, 0.62) * &scaling(0.22, 0.22, 0.22),
    );
    {
        let m = cluster_sphere.material_mut();
        m.color = Color::new(0.5, 0.82, 0.45);
        m.ambient = 0.08;
        m.diffuse = 0.75;
        m.specular = 0.35;
        m.shininess = 120.0;
    }

    let union_right = CSG::new(
        CSGOperation::Union,
        Box::new(union_lower),
        Box::new(cluster_sphere),
    );

    // --- Top: Difference( intersection, union_right )
    let mut root = CSG::new(
        CSGOperation::Difference,
        Box::new(intersection_shape),
        Box::new(union_right),
    );

    // Sit on floor: cube is [-1,1] in y; raise by 1 so bottom rests near y = 0
    root.set_transform(
        &(&translation(0.0, 1.05, 0.0) * &rotation_y(0.45)) * &rotation_x(-0.12),
    );

    world.objects.push(Box::new(root));

    world.lights = vec![
        PointLight::new(Tuple::point(-6.0, 9.0, -8.0), Color::new(1.0, 1.0, 1.0)),
        PointLight::new(Tuple::point(5.0, 4.0, -3.0), Color::new(0.2, 0.22, 0.28)),
    ];

    let mut camera = Camera::new(1000, 750, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(-4.2, 2.4, -6.5),
        &Tuple::point(0.0, 0.85, 0.0),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    let out = "media/images_ppm/csg_tree_demo.ppm";
    let ppm = camera.render(&world).canvas_to_ppm();
    std::fs::write(out, ppm).expect("write csg_tree_demo.ppm");
    println!("Saved to {out}");
}
