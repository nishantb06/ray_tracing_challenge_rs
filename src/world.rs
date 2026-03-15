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

pub fn shade_hit(world: &World, comps: &Computations) -> Color {
    let shadowed = world.is_shadowed(comps.over_point.clone());
    world
        .lights
        .iter()
        .fold(Color::new(0.0, 0.0, 0.0), |acc, light| {
            let c = lighting(
                comps.object.material(),
                light,
                &comps.point,
                &comps.eye_vector,
                &comps.normal_vector,
                shadowed,
            );
            &acc + &c
        })
}

pub fn color_at(world: &World, ray: &Ray) -> Color {
    let xs = world.intersect_world(ray);
    match xs.hit() {
        None => Color::new(0.0, 0.0, 0.0),
        Some(hit) => {
            let comps = prepare_computations(hit, ray);
            shade_hit(world, &comps)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::plane::Plane;
    use crate::ray::Ray;
    use crate::transformation::translation;

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
        let c = shade_hit(&w, &comps);
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
        let c = shade_hit(&w, &comps);
        assert!(c.is_equal(&Color::new(0.90498, 0.90498, 0.90498)));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = color_at(&w, &r);
        assert!(c.is_equal(&Color::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn the_color_when_a_ray_hits() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = color_at(&w, &r);
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
        let c = color_at(&w, &r);
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
        let c = shade_hit(&w, &comps);
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
}