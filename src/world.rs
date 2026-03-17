use crate::canvas::Color;
use crate::intersection::{Computations, Intersection, Intersections, prepare_computations};
use crate::light::PointLight;
use crate::material::lighting;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::transformation::scaling;
use crate::tuple::Tuple;
use crate::shape::Shape;

#[derive(Debug)]
pub struct World {
    pub objects: Vec<Box<dyn Shape>>,
    pub lights: Vec<PointLight>,
}

impl World {
    pub fn new() -> Self {
        World {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn default_world() -> Self {
        let light = PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        );

        let mut s1 = Sphere::new();
        s1.data.material.color = Color::new(0.8, 1.0, 0.6);
        s1.data.material.diffuse = 0.7;
        s1.data.material.specular = 0.2;

        let mut s2 = Sphere::new();
        s2.set_transform(scaling(0.5, 0.5, 0.5));

        World {
            objects: vec![Box::new(s1), Box::new(s2)],
            lights: vec![light],
        }
    }

    pub fn add_shape(&mut self, shape: impl Shape + 'static) {
        self.objects.push(Box::new(shape));
    }

    pub fn intersect_world(&self, ray: &Ray) -> Intersections<'_> {
        let mut all: Vec<Intersection> = Vec::new();
        for obj in &self.objects {
            let obj_xs = obj.intersect(ray);
            all.extend(obj_xs.data);
        }
        Intersections::new(all)
    }

    pub fn is_shadowed(&self, p: Tuple) -> bool {
        let v = &self.lights[0].position - &p;
        let magnitude = v.magnitude();
        let direction = v.normalize();
        let ray = Ray::new(p, direction);
        let xs = self.intersect_world(&ray);
        let hit = xs.hit();
        hit.is_some() && hit.unwrap().t < magnitude
    }
}

pub fn shade_hit(world: &World, comps: &Computations, remaining: i32) -> Color {
    let shadowed = world.is_shadowed(comps.over_point.clone());

    // surface ← lighting(...)
    let surface = world
        .lights
        .iter()
        .fold(Color::new(0.0, 0.0, 0.0), |acc, light| {
            let c = lighting(
                comps.object.material(),
                comps.object,
                light,
                &comps.point,
                &comps.eye_vector,
                &comps.normal_vector,
                shadowed,
            );
            &acc + &c
        });

    // reflected ← reflected_color(world, comps, remaining)
    let reflected = reflected_color(world, comps, remaining);

    // return surface + reflected
    &surface + &reflected
}

pub fn color_at(world: &World, ray: &Ray, remaining: i32) -> Color {
    let xs = world.intersect_world(ray);
    match xs.hit() {
        None => Color::new(0.0, 0.0, 0.0),
        Some(hit) => {
            let comps = prepare_computations(hit, ray);
            shade_hit(world, &comps, remaining)
        }
    }
}

pub fn reflected_color(world: &World, comps: &Computations, remaining: i32) -> Color {
    // stop recursion when remaining <= 0
    if remaining <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    // if material is not reflective, also return black
    let reflective = comps.object.material().reflective;
    if crate::utils::equal(reflective, 0.0) {
        return Color::new(0.0, 0.0, 0.0);
    }

    // reflect_ray ← ray(comps.over_point, comps.reflectv)
    let reflect_ray = Ray::new(comps.over_point.clone(), comps.reflectv.clone());

    // color ← color_at(world, reflect_ray, remaining - 1)
    let color = color_at(world, &reflect_ray, remaining - 1);

    // return color * comps.object.material.reflective
    &color * reflective
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::plane::Plane;
    use crate::ray::Ray;
    use crate::transformation::translation;
    use crate::utils::MAX_RECURSION_DEPTH;

    #[test]
    fn creating_a_world() {
        let w = World::new();
        assert!(w.objects.is_empty());
        assert!(w.lights.is_empty());
    }

    #[test]
    fn the_default_world() {
        let light = PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        );
        let w = World::default_world();
        assert_eq!(w.lights.len(), 1);
        assert!(w.lights[0].position.is_equal(&light.position));
        assert!(w.lights[0].intensity.is_equal(&light.intensity));
        assert_eq!(w.objects.len(), 2);
    }

    #[test]
    fn add_shape_adds_to_objects() {
        let mut w = World::new();
        w.add_shape(Sphere::new());
        w.add_shape(Plane::new());
        assert_eq!(w.objects.len(), 2);
    }

    #[test]
    fn intersect_a_world_with_a_ray() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = w.intersect_world(&r);
        assert_eq!(xs.count(), 4);
        assert!(crate::utils::equal(xs.data[0].t, 4.0));
        assert!(crate::utils::equal(xs.data[1].t, 4.5));
        assert!(crate::utils::equal(xs.data[2].t, 5.5));
        assert!(crate::utils::equal(xs.data[3].t, 6.0));
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(4.0, w.objects[0].as_ref());
        let comps = prepare_computations(&i, &r);
        let c = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.38066, 0.47583, 0.2855)));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default_world();
        w.lights = vec![PointLight::new(
            Tuple::point(0.0, 0.25, 0.0),
            Color::new(1.0, 1.0, 1.0),
        )];
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(0.5, w.objects[1].as_ref());
        let comps = prepare_computations(&i, &r);
        let c = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.90498, 0.90498, 0.90498)));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = color_at(&w, &r, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn the_color_when_a_ray_hits() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = color_at(&w, &r, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.38066, 0.47583, 0.2855)));
    }

    #[test]
    fn the_color_with_an_intersection_behind_the_ray() {
        let mut w = World::default_world();
        // mutate via data directly since we need concrete access
        w.objects[0].shape_data_mut().material.ambient = 1.0;
        w.objects[1].shape_data_mut().material.ambient = 1.0;
        let inner_color = w.objects[1].material().color.clone();
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let c = color_at(&w, &r, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&inner_color));
    }

    #[test]
    fn no_shadow_when_nothing_is_colinear_with_point_and_light() {
        let w = World::default_world();
        assert!(!w.is_shadowed(Tuple::point(0.0, 10.0, 0.0)));
    }

    #[test]
    fn shadow_when_object_is_between_point_and_light() {
        let w = World::default_world();
        assert!(w.is_shadowed(Tuple::point(10.0, -10.0, 10.0)));
    }

    #[test]
    fn no_shadow_when_object_is_behind_light() {
        let w = World::default_world();
        assert!(!w.is_shadowed(Tuple::point(-20.0, 20.0, -20.0)));
    }

    #[test]
    fn no_shadow_when_object_is_behind_point() {
        let w = World::default_world();
        assert!(!w.is_shadowed(Tuple::point(-2.0, 2.0, -2.0)));
    }

    #[test]
    fn shade_hit_given_an_intersection_in_shadow() {
        let mut w = World::new();
        w.lights = vec![PointLight::new(
            Tuple::point(0.0, 0.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        )];
        w.add_shape(Sphere::new());
        let mut s2 = Sphere::new();
        s2.set_transform(translation(0.0, 0.0, 10.0));
        w.add_shape(s2);
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(4.0, w.objects[1].as_ref());
        let comps = prepare_computations(&i, &r);
        let c = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.1, 0.1, 0.1)));
    }

    #[test]
    fn the_hit_should_offset_the_point() {
        let mut shape = Sphere::new();
        shape.set_transform(translation(0.0, 0.0, 1.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(5.0, &shape as &dyn Shape);
        let comps = prepare_computations(&i, &r);
        assert!(comps.over_point.z < -crate::utils::EPSILON / 2.0);
        assert!(comps.point.z > comps.over_point.z);
    }

    #[test]
    fn the_reflected_color_for_a_nonreflective_material() {
        // Given w ← default_world()
        let mut w = World::default_world();

        // And shape ← the second object in w
        // And shape.material.ambient ← 1
        w.objects[1].shape_data_mut().material.ambient = 1.0;
        let shape = w.objects[1].as_ref();

        // And r ← ray(point(0, 0, 0), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, 0.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );

        // And i ← intersection(1, shape)
        let i = Intersection::new(1.0, shape);

        // When comps ← prepare_computations(i, r)
        let comps = prepare_computations(&i, &r);

        // And color ← reflected_color(w, comps)
        let color = reflected_color(&w, &comps, MAX_RECURSION_DEPTH);

        // Then color = color(0, 0, 0)
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn the_reflected_color_for_a_reflective_material() {
        let mut w = World::default_world();

        // create and configure the plane
        let mut shape = Plane::new();
        shape.shape_data_mut().material.reflective = 0.5;
        shape.set_transform(translation(0.0, -1.0, 0.0));

        // And shape is added to w
        w.add_shape(shape);

        // now get a reference to that shape from the world
        let shape_ref: &dyn Shape = w.objects.last().unwrap().as_ref();

        // And r ← ray(point(0, 0, -3), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -3.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );

        // And i ← intersection(√2, shape)
        let i = Intersection::new(std::f64::consts::SQRT_2, shape_ref);

        let comps = prepare_computations(&i, &r);
        let color = reflected_color(&w, &comps, MAX_RECURSION_DEPTH);
        let expected = Color::new(0.1903323, 0.237915, 0.142749);
        assert!(color.is_equal(&expected));
    }

    #[test]
    fn shade_hit_with_a_reflective_material() {
        // Given w ← default_world()
        let mut w = World::default_world();

        // And shape ← plane() with:
        // | material.reflective | 0.5 |
        // | transform           | translation(0, -1, 0) |
        let mut shape = Plane::new();
        shape.shape_data_mut().material.reflective = 0.5;
        shape.set_transform(translation(0.0, -1.0, 0.0));

        // And shape is added to w
        w.add_shape(shape);

        // get a reference to the added plane
        let shape_ref: &dyn Shape = w.objects.last().unwrap().as_ref();

        // And r ← ray(point(0, 0, -3), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -3.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );

        // And i ← intersection(√2, shape)
        let i = Intersection::new(std::f64::consts::SQRT_2, shape_ref);

        // When comps ← prepare_computations(i, r)
        let comps = prepare_computations(&i, &r);

        // And color ← shade_hit(w, comps)
        let color = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);

        // Then color = color(0.87677, 0.92436, 0.82918)
        let expected = Color::new(0.87675, 0.92434, 0.82917);
        dbg!(&color);
        assert!(color.is_equal(&expected));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces_terminates() {
        // Given w ← world()
        let mut w = World::new();

        // And w.light ← point_light(point(0, 0, 0), color(1, 1, 1))
        w.lights = vec![PointLight::new(
            Tuple::point(0.0, 0.0, 0.0),
            Color::new(1.0, 1.0, 1.0),
        )];

        // And lower ← plane() with reflective = 1, translation(0, -1, 0)
        let mut lower = Plane::new();
        lower.shape_data_mut().material.reflective = 1.0;
        lower.set_transform(translation(0.0, -1.0, 0.0));
        w.add_shape(lower);

        // And upper ← plane() with reflective = 1, translation(0, 1, 0)
        let mut upper = Plane::new();
        upper.shape_data_mut().material.reflective = 1.0;
        upper.set_transform(translation(0.0, 1.0, 0.0));
        w.add_shape(upper);

        // And r ← ray(point(0, 0, 0), vector(0, 1, 0))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
        );

        // Then color_at(w, r) should terminate successfully
        let _ = color_at(&w, &r, MAX_RECURSION_DEPTH); // test passes as long as this returns without panic
    }

    #[test]
    fn the_reflected_color_at_the_max_recursive_depth() {
        // Given w ← default_world()
        let mut w = World::default_world();

        // And shape ← plane() with:
        // | material.reflective | 0.5 |
        // | transform           | translation(0, -1, 0) |
        let mut shape = Plane::new();
        shape.shape_data_mut().material.reflective = 0.5;
        shape.set_transform(translation(0.0, -1.0, 0.0));

        // And shape is added to w
        w.add_shape(shape);

        // get a reference to the added plane
        let shape_ref: &dyn Shape = w.objects.last().unwrap().as_ref();

        // And r ← ray(point(0, 0, -3), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -3.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );

        // And i ← intersection(√2, shape)
        let i = Intersection::new(std::f64::consts::SQRT_2, shape_ref);

        // When comps ← prepare_computations(i, r)
        let comps = prepare_computations(&i, &r);

        // And color ← reflected_color(w, comps, 0)
        let color = reflected_color(&w, &comps, 0);

        // Then color = color(0, 0, 0)
        let expected = Color::new(0.0, 0.0, 0.0);
        assert!(color.is_equal(&expected));
    }
}