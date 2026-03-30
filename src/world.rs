use crate::canvas::Color;
use crate::intersection::{Computations, Intersection, Intersections, prepare_computations, schlick};
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

    pub fn resolve_shape(&self, id: u64) -> Option<&dyn Shape> {
        for object in &self.objects {
            if object.id() == id {
                return Some(object.as_ref());
            }
            if let Some(found) = object.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn is_shadowed_light(&self, p: Tuple, light: &PointLight) -> bool {
        let v = &light.position - &p;
        let magnitude = v.magnitude();
        let direction = v.normalize();
        let ray = Ray::new(p, direction);
        let xs = self.intersect_world(&ray);
        let hit = xs.hit();
        hit.is_some() && hit.unwrap().t < magnitude
    }

    pub fn is_shadowed(&self, p: Tuple) -> bool {
        if self.lights.is_empty() {
            return false;
        }
        self.lights
            .iter()
            .all(|light| self.is_shadowed_light(p.clone(), light))
    }
}

pub fn shade_hit(world: &World, comps: &Computations, remaining: i32) -> Color {
    // surface ← lighting(...)
    let surface = world
        .lights
        .iter()
        .fold(Color::new(0.0, 0.0, 0.0), |acc, light| {
            let shadowed = world.is_shadowed_light(comps.over_point.clone(), light);
            let c = lighting(
                comps.object.material(),
                comps.object,
                light,
                &comps.over_point,   // use over_point as in the book
                &comps.eye_vector,
                &comps.normal_vector,
                shadowed,
            );
            &acc + &c
        });

    let reflected = reflected_color(world, comps, remaining);
    let refracted = refracted_color(world, comps, remaining);
    let material = comps.object.material();

    if material.reflective > 0.0 && material.transparency > 0.0 {
        let reflectance = schlick(comps);
        &surface + &(&(&reflected * reflectance) + &(&refracted * (1.0 - reflectance)))
    } else {
        &(&surface + &reflected) + &refracted
    }
}

pub fn color_at(world: &World, ray: &Ray, remaining: i32) -> Color {
    let xs = world.intersect_world(ray);
    match xs.hit() {
        None => Color::new(0.0, 0.0, 0.0),
        Some(hit) => {
            let comps = prepare_computations(hit, ray, &xs, &|id| world.resolve_shape(id));
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

pub fn refracted_color(world: &World, comps: &Computations, remaining: i32) -> Color {
    let transparency = comps.object.material().transparency;

    // If the material is opaque or we've hit the recursion limit, return black
    if transparency == 0.0 || remaining <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    // Snell's law pieces
    let n_ratio = comps.n1 / comps.n2;
    let cos_i = comps.eye_vector.dot(&comps.normal_vector);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);

    // Total internal reflection: no refraction, return black
    if sin2_t > 1.0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    // Find cos(theta_t) via trigonometric identity
    let cos_t = (1.0 - sin2_t).sqrt();

    // Compute the direction of the refracted ray
    let term1 = &comps.normal_vector * (n_ratio * cos_i - cos_t);
    let term2 = &comps.eye_vector * n_ratio;
    let direction = &term1 - &term2;

    // Create the refracted ray
    let refract_ray = Ray::new(comps.under_point.clone(), direction);

    // Find the color of the refracted ray, scaled by transparency
    let color = color_at(world, &refract_ray, remaining - 1);
    &color * transparency
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::Group;
    use crate::plane::Plane;
    use crate::ray::Ray;
    use crate::transformation::{scaling, translation};
    use crate::utils::MAX_RECURSION_DEPTH;
    
    fn no_parent<'a>(_id: u64) -> Option<&'a dyn Shape> {
        None
    }

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
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
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
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
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
    fn is_shadowed_for_light_returns_false_when_path_clear() {
        let mut w = World::new();
        w.add_shape(Sphere::new());
        let p = Tuple::point(0.0, 0.0, -3.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -3.0), Color::new(1.0, 1.0, 1.0));
        assert!(!w.is_shadowed_light(p, &light));
    }

    #[test]
    fn is_shadowed_for_light_returns_true_when_occluded() {
        let mut w = World::new();
        w.add_shape(Sphere::new());
        let p = Tuple::point(0.0, 0.0, -3.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, 3.0), Color::new(1.0, 1.0, 1.0));
        assert!(w.is_shadowed_light(p, &light));
    }

    #[test]
    fn is_shadowed_returns_false_when_at_least_one_light_is_visible() {
        let mut w = World::new();
        w.add_shape(Sphere::new());
        w.lights = vec![
            PointLight::new(Tuple::point(0.0, 10.0, -3.0), Color::new(1.0, 1.0, 1.0)),
            PointLight::new(Tuple::point(0.0, 0.0, 3.0), Color::new(1.0, 1.0, 1.0)),
        ];
        assert!(!w.is_shadowed(Tuple::point(0.0, 0.0, -3.0)));
    }

    #[test]
    fn is_shadowed_returns_true_when_all_lights_are_occluded() {
        let mut w = World::new();
        w.add_shape(Sphere::new());
        w.lights = vec![
            PointLight::new(Tuple::point(0.0, 0.0, 3.0), Color::new(1.0, 1.0, 1.0)),
            PointLight::new(Tuple::point(0.0, 0.0, 6.0), Color::new(1.0, 1.0, 1.0)),
        ];
        assert!(w.is_shadowed(Tuple::point(0.0, 0.0, -3.0)));
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
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
        let c = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(0.1, 0.1, 0.1)));
    }

    #[test]
    fn shade_hit_with_mixed_light_visibility_uses_per_light_shadowing() {
        let mut w = World::new();
        w.lights = vec![
            PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)),
            PointLight::new(Tuple::point(0.0, 0.0, 5.0), Color::new(1.0, 1.0, 1.0)),
        ];
        w.add_shape(Sphere::new());
        let mut s2 = Sphere::new();
        s2.set_transform(translation(0.0, 0.0, 10.0));
        w.add_shape(s2);
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(4.0, w.objects[1].as_ref());
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
        let c = shade_hit(&w, &comps, MAX_RECURSION_DEPTH);
        assert!(c.is_equal(&Color::new(2.0, 2.0, 2.0)));
    }

    #[test]
    fn the_hit_should_offset_the_point() {
        let mut shape = Sphere::new();
        shape.set_transform(translation(0.0, 0.0, 1.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(5.0, &shape as &dyn Shape);
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
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
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);

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

        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);
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
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);

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
        let comps = prepare_computations(&i, &r,&Intersections::new(vec![i.clone()]), &no_parent);

        // And color ← reflected_color(w, comps, 0)
        let color = reflected_color(&w, &comps, 0);

        // Then color = color(0, 0, 0)
        let expected = Color::new(0.0, 0.0, 0.0);
        assert!(color.is_equal(&expected));
    }

    #[test]
    fn the_refracted_color_with_an_opaque_surface() {
        // Given w ← default_world()
        let w = World::default_world();
    
        // And shape ← the first object in w
        let shape = w.objects[0].as_ref();
    
        // And r ← ray(point(0, 0, -5), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -5.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And xs ← intersections(4:shape, 6:shape)
        let i1 = Intersection::new(4.0, shape);
        let i2 = Intersection::new(6.0, shape);
        let xs = Intersections::new(vec![i1, i2]);
    
        // When comps ← prepare_computations(xs[0], r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);
    
        // And c ← refracted_color(w, comps, 5)
        let c = refracted_color(&w, &comps, 5);
    
        // Then c = color(0, 0, 0)
        assert!(c.is_equal(&Color::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn the_refracted_color_at_the_max_recursive_depth() {
        // Given w ← default_world()
        let mut w = World::default_world();
    
        // And shape ← the first object in w
        let shape_mut = w.objects[0].shape_data_mut();
    
        // And shape has:
        // | material.transparency     | 1.0 |
        // | material.refractive_index | 1.5 |
        shape_mut.material.transparency = 1.0;
        shape_mut.material.refractive_index = 1.5;
    
        let shape = w.objects[0].as_ref();
    
        // And r ← ray(point(0, 0, -5), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -5.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And xs ← intersections(4:shape, 6:shape)
        let i1 = Intersection::new(4.0, shape);
        let i2 = Intersection::new(6.0, shape);
        let xs = Intersections::new(vec![i1, i2]);
    
        // When comps ← prepare_computations(xs[0], r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);
    
        // And c ← refracted_color(w, comps, 0)
        let c = refracted_color(&w, &comps, 0);
    
        // Then c = color(0, 0, 0)
        let expected = Color::new(0.0, 0.0, 0.0);
        assert!(c.is_equal(&expected));
    }

    #[test]
    fn the_refracted_color_under_total_internal_reflection() {
        // Given w ← default_world()
        let mut w = World::default_world();

        // And shape ← the first object in w
        let shape_data = w.objects[0].shape_data_mut();

        // And shape has:
        // | material.transparency     | 1.0 |
        // | material.refractive_index | 1.5 |
        shape_data.material.transparency = 1.0;
        shape_data.material.refractive_index = 1.5;

        let shape = w.objects[0].as_ref();

        // And r ← ray(point(0, 0, √2/2), vector(0, 1, 0))
        let half_sqrt2 = std::f64::consts::SQRT_2 / 2.0;
        let r = Ray::new(
            Tuple::point(0.0, 0.0, half_sqrt2),
            Tuple::vector(0.0, 1.0, 0.0),
        );

        // And xs ← intersections(-√2/2:shape, √2/2:shape)
        let i1 = Intersection::new(-half_sqrt2, shape);
        let i2 = Intersection::new(half_sqrt2, shape);
        let xs = Intersections::new(vec![i1, i2]);

        // When comps ← prepare_computations(xs[1], r, xs)
        let comps = prepare_computations(&xs.data[1], &r, &xs, &no_parent);

        // And c ← refracted_color(w, comps, 5)
        let c = refracted_color(&w, &comps, 5);

        // Then c = color(0, 0, 0)
        let expected = Color::new(0.0, 0.0, 0.0);
        assert!(c.is_equal(&expected));
    }

    #[test]
    fn the_refracted_color_with_a_refracted_ray() {
        use crate::pattern::test_support::TestPattern;

        // Given w ← default_world()
        let mut w = World::default_world();

        // Configure A (first object)
        {
            let a_data = w.objects[0].shape_data_mut();
            a_data.material.ambient = 1.0;
            a_data.material.pattern = Some(Box::new(TestPattern::new()));
        }

        // Configure B (second object)
        {
            let b_data = w.objects[1].shape_data_mut();
            b_data.material.transparency = 1.0;
            b_data.material.refractive_index = 1.5;
        }

        // Now take immutable references after all mutation is done
        let a = w.objects[0].as_ref();
        let b = w.objects[1].as_ref();

        // And r ← ray(point(0, 0, 0.1), vector(0, 1, 0))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, 0.1),
            Tuple::vector(0.0, 1.0, 0.0),
        );

        // And xs ← intersections(-0.9899:A, -0.4899:B, 0.4899:B, 0.9899:A)
        let xs = Intersections::new(vec![
            Intersection::new(-0.9899, a),
            Intersection::new(-0.4899, b),
            Intersection::new(0.4899, b),
            Intersection::new(0.9899, a),
        ]);

        // When comps ← prepare_computations(xs[2], r, xs)
        let comps = prepare_computations(&xs.data[2], &r, &xs, &no_parent);

        // And c ← refracted_color(w, comps, 5)
        let c = refracted_color(&w, &comps, 5);
        // Then c = color(0, 0.99888, 0.04725)
        let expected = Color::new(0.0, 0.99888, 0.047219);
        assert!(c.is_equal(&expected));
    }

    #[test]
    fn shade_hit_with_a_transparent_material() {
        use crate::plane::Plane;
        use crate::sphere::Sphere;
        use crate::transformation::translation;
        use crate::tuple::Tuple;
        use crate::intersection::{Intersection, Intersections};

        // Given w ← default_world()
        let mut w = World::default_world();

        // And floor ← plane() with:
        // | transform              | translation(0, -1, 0) |
        // | material.transparency  | 0.5                   |
        // | material.refractive_index | 1.5                |
        let mut floor = Plane::new();
        {
            let data = floor.shape_data_mut();
            data.material.transparency = 0.5;
            data.material.refractive_index = 1.5;
        }
        floor.set_transform(translation(0.0, -1.0, 0.0));

        // And floor is added to w
        w.add_shape(floor);

        // And ball ← sphere() with:
        // | material.color   | (1, 0, 0)               |
        // | material.ambient | 0.5                     |
        // | transform        | translation(0, -3.5, -0.5) |
        let mut ball = Sphere::new();
        {
            let data = ball.shape_data_mut();
            data.material.color = Color::new(1.0, 0.0, 0.0);
            data.material.ambient = 0.5;
        }
        ball.set_transform(translation(0.0, -3.5, -0.5));

        // And ball is added to w
        w.add_shape(ball);

        // floor is the third object: index 2 (0: s1, 1: s2, 2: floor, 3: ball)
        let floor_ref: &dyn Shape = w.objects[2].as_ref();

        // And r ← ray(point(0, 0, -3), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -3.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );

        // And xs ← intersections(√2:floor)
        let i = Intersection::new(std::f64::consts::SQRT_2, floor_ref);
        let xs = Intersections::new(vec![i]);

        // When comps ← prepare_computations(xs[0], r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);

        // And color ← shade_hit(w, comps, 5)
        let color = shade_hit(&w, &comps, 5);

        // Then color = color(0.93642, 0.68642, 0.68642)
        let expected = Color::new(0.93642, 0.68642, 0.68642);
        assert!(color.is_equal(&expected));
    }

    #[test]
    fn shade_hit_with_a_reflective_transparent_material() {
        use crate::plane::Plane;
        use crate::sphere::Sphere;
        use crate::transformation::translation;
        use crate::tuple::Tuple;
        use crate::intersection::{Intersection, Intersections};
    
        // Given w ← default_world()
        let mut w = World::default_world();
    
        // And r ← ray(point(0, 0, -3), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -3.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );
    
        // And floor ← plane() with:
        // | transform            | translation(0, -1, 0) |
        // | material.reflective  | 0.5                   |
        // | material.transparency| 0.5                   |
        // | material.refractive_index | 1.5             |
        let mut floor = Plane::new();
        {
            let data = floor.shape_data_mut();
            data.material.reflective = 0.5;
            data.material.transparency = 0.5;
            data.material.refractive_index = 1.5;
        }
        floor.set_transform(translation(0.0, -1.0, 0.0));
    
        // And floor is added to w
        w.add_shape(floor);
    
        // And ball ← sphere() with:
        // | material.color   | (1, 0, 0)               |
        // | material.ambient | 0.5                     |
        // | transform        | translation(0, -3.5, -0.5) |
        let mut ball = Sphere::new();
        {
            let data = ball.shape_data_mut();
            data.material.color = Color::new(1.0, 0.0, 0.0);
            data.material.ambient = 0.5;
        }
        ball.set_transform(translation(0.0, -3.5, -0.5));
    
        // And ball is added to w
        w.add_shape(ball);
    
        // floor is the third object: index 2 (0: s1, 1: s2, 2: floor, 3: ball)
        let floor_ref: &dyn Shape = w.objects[2].as_ref();
    
        // And xs ← intersections(√2:floor)
        let i = Intersection::new(std::f64::consts::SQRT_2, floor_ref);
        let xs = Intersections::new(vec![i]);
    
        // When comps ← prepare_computations(xs[0], r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);
    
        // And color ← shade_hit(w, comps, 5)
        let color = shade_hit(&w, &comps, 5);
    
        // Then color = color(0.93391, 0.69643, 0.69243)
        let expected = Color::new(0.93391, 0.69643, 0.69243);
        assert!(color.is_equal(&expected));
    }

    #[test]
    fn color_at_matches_direct_transform_for_grouped_child_shape() {
        let mut grouped_world = World::new();
        grouped_world.lights = vec![PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        )];
    
        let mut group = Group::new();
        let group_transform = scaling(1.0, 0.5, 1.0);
        group.set_transform(group_transform.clone());
    
        let child_transform = translation(0.0, 0.0, 1.0);
        let mut child = Sphere::new(); // owned (no leak needed)
        child.set_transform(child_transform.clone());
        child.material_mut().color = Color::new(0.2, 0.8, 1.0);
        child.material_mut().diffuse = 0.7;
        child.material_mut().specular = 0.3;
        // child.shape_data_mut().parent = Some(group.id()); // no need anymore after new change
    
        group.add_child(Box::new(child)); // Box<Sphere> -> Box<dyn Shape>
        grouped_world.add_shape(group);
    
        let mut direct_world = World::new();
        direct_world.lights = vec![PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        )];
    
        let mut direct_sphere = Sphere::new();
        direct_sphere.set_transform(&group_transform * &child_transform);
        direct_sphere.material_mut().color = Color::new(0.2, 0.8, 1.0);
        direct_sphere.material_mut().diffuse = 0.7;
        direct_sphere.material_mut().specular = 0.3;
        direct_world.add_shape(direct_sphere);
    
        let ray = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let grouped_color = color_at(&grouped_world, &ray, MAX_RECURSION_DEPTH);
        let direct_color = color_at(&direct_world, &ray, MAX_RECURSION_DEPTH);
    
        assert!(grouped_color.is_equal(&direct_color));
    }

    #[test]
    fn is_shadowed_is_false_when_at_least_one_light_is_visible() {
        let mut w = World::new();
        // One simple sphere in the scene
        w.add_shape(Sphere::new());
        // First light is visible from p, second is blocked by the sphere.
        w.lights = vec![
            PointLight::new(
                Tuple::point(0.0, 10.0, -3.0),
                Color::new(1.0, 1.0, 1.0),
            ),
            PointLight::new(
                Tuple::point(0.0, 0.0, 3.0),
                Color::new(1.0, 1.0, 1.0),
            ),
        ];
        let p = Tuple::point(0.0, 0.0, -3.0);
        // Expected: false for aggregate semantics ("all lights blocked" only).
        assert!(!w.is_shadowed(p));
    }
}