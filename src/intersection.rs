use std::fmt;
use crate::ray::Ray;
use crate::shape::shape_normal_at;
use crate::tuple::Tuple;
use crate::utils::EPSILON;
use crate::shape::Shape;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a dyn Shape,
}

impl fmt::Debug for Intersection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Intersection")
            .field("t", &self.t)
            .field("object", &format!("Shape#{}", self.object.id()))
            .finish()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Intersections<'a> {
    pub data: Vec<Intersection<'a>>,
}

#[allow(dead_code)]
impl<'a> Intersection<'a> {
    pub fn new(t: f64, object: &'a dyn Shape) -> Self {
        Intersection { t, object }
    }
}

pub struct Computations<'a> {
    pub t: f64,
    pub object: &'a dyn Shape,
    pub point: Tuple,
    pub eye_vector: Tuple,
    pub normal_vector: Tuple,
    pub inside: bool,
    pub over_point: Tuple,
    pub reflectv: Tuple,
    pub n1: f64,
    pub n2: f64,
    pub under_point: Tuple,
}

// precomputes the point (in world space) where the intersection occurred,
// the eye vector (pointing back toward the eye, or camera), and the normal vector.
pub fn prepare_computations<'a>(
    intersection: &'a Intersection<'a>,
    ray: &Ray,
    xs: &Intersections<'a>,
    resolve_parent: &impl Fn(u64) -> Option<&'a dyn Shape>,
) -> Computations<'a> {
    // existing geometric precomputations
    let point = ray.position(intersection.t);
    let mut normal_v = shape_normal_at(intersection.object, resolve_parent, &point);
    let eye_v = -&ray.direction;
    let mut inside = false;

    if normal_v.dot(&eye_v) < 0.0 {
        inside = true;
        normal_v = -&normal_v;
    }

    let over_point = &point + &(&normal_v * EPSILON);
    let under_point = &point - &(&normal_v * EPSILON);
    let reflectv = ray.direction.reflect(&normal_v);

    // new: refractive indices via containers algorithm
    let mut n1 = 1.0;
    let mut n2 = 1.0;
    let mut containers: Vec<&'a dyn Shape> = Vec::new();

    for i in &xs.data {
        // if this intersection is the hit, set n1 from current containers
        if std::ptr::eq(i, intersection) {
            n1 = containers
                .last()
                .map(|o| o.material().refractive_index)
                .unwrap_or(1.0);
        }

        // update containers: exiting if already present, otherwise entering
        if let Some(pos) = containers.iter().position(|o| o.id() == i.object.id()) {
            containers.remove(pos);
        } else {
            containers.push(i.object);
        }

        // if this intersection is the hit, set n2 and stop
        if std::ptr::eq(i, intersection) {
            n2 = containers
                .last()
                .map(|o| o.material().refractive_index)
                .unwrap_or(1.0);
            break;
        }
    }

    Computations {
        t: intersection.t,
        object: intersection.object,
        point,
        eye_vector: eye_v,
        normal_vector: normal_v,
        inside,
        over_point,
        reflectv,
        n1,
        n2,
        under_point,
    }
}

#[allow(dead_code)]
impl<'a> Intersections<'a> {
    pub fn new(mut items: Vec<Intersection<'a>>) -> Self {
        items.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        Intersections { data: items }
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn hit(&self) -> Option<&Intersection<'a>> {
        self.data.iter().find(|i| i.t >= 0.0)
    }
}

pub fn schlick(comps: &Computations) -> f64 {
    // find the cosine of the angle between the eye and normal vectors
    let mut cos = comps.eye_vector.dot(&comps.normal_vector);

    // total internal reflection can only occur if n1 > n2
    if comps.n1 > comps.n2 {
        let n = comps.n1 / comps.n2;
        let sin2_t = n * n * (1.0 - cos * cos);

        // return 1.0 if we have total internal reflection
        if sin2_t > 1.0 {
            return 1.0;
        }

        // compute cosine of theta_t using trig identity
        let cos_t = (1.0 - sin2_t).sqrt();

        // when n1 > n2, use cos(theta_t) instead
        cos = cos_t;
    }

    let r0 = ((comps.n1 - comps.n2) / (comps.n1 + comps.n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::Sphere;
    
    fn no_parent<'a>(_id: u64) -> Option<&'a dyn Shape> {
        None
    }

    #[test]
    fn an_intersection_encapsulates_t_and_object() {
        let s = Sphere::new();
        let i = Intersection::new(3.5, &s);
        assert!(crate::utils::equal(i.t, 3.5));
        assert_eq!(i.object.id(), s.data.id);
    }

    #[test]
    fn aggregating_intersections() {
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i1, i2]);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 1.0));
        assert!(crate::utils::equal(xs.data[1].t, 2.0));
    }

    #[test]
    fn the_hit_when_all_intersections_have_positive_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 1.0));
    }

    #[test]
    fn the_hit_when_some_intersections_have_negative_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(1.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 1.0));
    }

    #[test]
    fn the_hit_when_all_intersections_have_negative_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(-2.0, &s);
        let i2 = Intersection::new(-1.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit();
        assert!(i.is_none());
    }

    #[test]
    fn the_hit_is_always_the_lowest_nonnegative_intersection() {
        let s = Sphere::new();
        let i1 = Intersection::new(5.0, &s);
        let i2 = Intersection::new(7.0, &s);
        let i3 = Intersection::new(-3.0, &s);
        let i4 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i1, i2, i3, i4]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 2.0));
    }

    #[test]
    fn precomputing_the_state_of_an_intersection() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(4.0, &shape);
        let comps = prepare_computations(&i, &r,&Intersections::new(vec![i.clone()]), &no_parent);
        assert!(crate::utils::equal(comps.t, i.t));
        assert_eq!(comps.object.id(), i.object.id());
        assert!(comps.point.is_equal(&Tuple::point(0.0, 0.0, -1.0)));
        assert!(comps.eye_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
        assert!(comps.normal_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
    }

    #[test]
    fn the_hit_when_intersection_occurs_on_the_outside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(4.0, &shape);
        let comps = prepare_computations(&i, &r,&Intersections::new(vec![i.clone()]), &no_parent);
        assert!(!comps.inside);
    }

    #[test]
    fn the_hit_when_intersection_occurs_on_the_inside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(1.0, &shape);
        let comps = prepare_computations(&i, &r,&Intersections::new(vec![i.clone()]), &no_parent);
        assert!(comps.point.is_equal(&Tuple::point(0.0, 0.0, 1.0)));
        assert!(comps.eye_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
        assert!(comps.inside);
        assert!(comps.normal_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
    }

    use crate::plane::Plane;

    #[test]
    fn precomputing_the_reflection_vector() {
        // shape ← plane()
        let shape = Plane::new();

        // r ← ray(point(0, 1, -1), vector(0, -√2/2, √2/2))
        let r = Ray::new(
            Tuple::point(0.0, 1.0, -1.0),
            Tuple::vector(
                0.0,
                -std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ),
        );

        // i ← intersection(√2, shape)
        let i = Intersection::new(std::f64::consts::SQRT_2, &shape);

        // comps ← prepare_computations(i, r)
        let comps = prepare_computations(&i, &r, &Intersections::new(vec![i.clone()]), &no_parent);

        // Then comps.reflectv = vector(0, √2/2, √2/2)
        let expected = Tuple::vector(
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        );
        assert!(comps.reflectv.is_equal(&expected));
    }

    #[test]
    fn finding_n1_and_n2_at_various_intersections() {
        use crate::sphere::Sphere;
        use crate::transformation::{scaling, translation};
        use crate::tuple::Tuple;
        use crate::ray::Ray;
    
        // Given A ← glass_sphere() with:
        // | transform              | scaling(2, 2, 2) |
        // | material.refractive_index | 1.5          |
        let mut a = Sphere::glass_sphere();
        a.set_transform(scaling(2.0, 2.0, 2.0));
        a.material_mut().refractive_index = 1.5;
    
        // And B ← glass_sphere() with:
        // | transform              | translation(0, 0, -0.25) |
        // | material.refractive_index | 2.0                  |
        let mut b = Sphere::glass_sphere();
        b.set_transform(translation(0.0, 0.0, -0.25));
        b.material_mut().refractive_index = 2.0;
    
        // And C ← glass_sphere() with:
        // | transform              | translation(0, 0, 0.25) |
        // | material.refractive_index | 2.5                 |
        let mut c = Sphere::glass_sphere();
        c.set_transform(translation(0.0, 0.0, 0.25));
        c.material_mut().refractive_index = 2.5;
    
        // And r ← ray(point(0, 0, -4), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -4.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And xs ← intersections(2:A, 2.75:B, 3.25:C, 4.75:B, 5.25:C, 6:A)
        let xs = Intersections::new(vec![
            Intersection::new(2.0,   &a),
            Intersection::new(2.75,  &b),
            Intersection::new(3.25,  &c),
            Intersection::new(4.75,  &b),
            Intersection::new(5.25,  &c),
            Intersection::new(6.0,   &a),
        ]);
    
        // Examples table from the book
        let examples = [
            (0, 1.0, 1.5),
            (1, 1.5, 2.0),
            (2, 2.0, 2.5),
            (3, 2.5, 2.5),
            (4, 2.5, 1.5),
            (5, 1.5, 1.0),
        ];
    
        for (index, expected_n1, expected_n2) in examples {
            let i = &xs.data[index];
            let comps = prepare_computations(i, &r, &xs, &no_parent);
            assert!(
                crate::utils::equal(comps.n1, expected_n1),
                "index {}: expected n1 = {}, got {}",
                index, expected_n1, comps.n1
            );
            assert!(
                crate::utils::equal(comps.n2, expected_n2),
                "index {}: expected n2 = {}, got {}",
                index, expected_n2, comps.n2
            );
        }
    }

    #[test]
    fn the_under_point_is_offset_below_the_surface() {
        use crate::ray::Ray;
        use crate::sphere::Sphere;
        use crate::transformation::translation;
        use crate::tuple::Tuple;
        use crate::utils::EPSILON;
        use crate::intersection::{Intersection, Intersections, prepare_computations};
    
        // Given r ← ray(point(0, 0, -5), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, -5.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And shape ← glass_sphere() with:
        // | transform | translation(0, 0, 1) |
        let mut shape = Sphere::glass_sphere();
        shape.set_transform(translation(0.0, 0.0, 1.0));
    
        // And i ← intersection(5, shape)
        let i = Intersection::new(5.0, &shape as &dyn Shape);
    
        // And xs ← intersections(i)
        let xs = Intersections::new(vec![i]);
    
        // When comps ← prepare_computations(i, r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);
    
        // Then comps.under_point.z > EPSILON/2
        assert!(comps.under_point.z > EPSILON / 2.0);
    
        // And comps.point.z < comps.under_point.z
        assert!(comps.point.z < comps.under_point.z);
    }

    #[test]
    fn the_schlick_approximation_under_total_internal_reflection() {
        use crate::sphere::Sphere;
        use crate::ray::Ray;
        use crate::tuple::Tuple;

        // Given shape ← glass_sphere()
        let shape = Sphere::glass_sphere();

        // And r ← ray(point(0, 0, √2/2), vector(0, 1, 0))
        let half_sqrt2 = std::f64::consts::SQRT_2 / 2.0;
        let r = Ray::new(
            Tuple::point(0.0, 0.0, half_sqrt2),
            Tuple::vector(0.0, 1.0, 0.0),
        );

        // And xs ← intersections(-√2/2:shape, √2/2:shape)
        let xs = Intersections::new(vec![
            Intersection::new(-half_sqrt2, &shape),
            Intersection::new(half_sqrt2, &shape),
        ]);

        // When comps ← prepare_computations(xs[1], r, xs)
        let comps = prepare_computations(&xs.data[1], &r, &xs, &no_parent);

        // And reflectance ← schlick(comps)
        let reflectance = schlick(&comps);

        // Then reflectance = 1.0
        assert!(crate::utils::equal(reflectance, 1.0));
    }

    #[test]
    fn the_schlick_approximation_with_a_perpendicular_viewing_angle() {
        use crate::sphere::Sphere;
        use crate::ray::Ray;
        use crate::tuple::Tuple;
    
        // Given shape ← glass_sphere()
        let shape = Sphere::glass_sphere();
    
        // And r ← ray(point(0, 0, 0), vector(0, 1, 0))
        let r = Ray::new(
            Tuple::point(0.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
        );
    
        // And xs ← intersections(-1:shape, 1:shape)
        let xs = Intersections::new(vec![
            Intersection::new(-1.0, &shape),
            Intersection::new(1.0, &shape),
        ]);
    
        // When comps ← prepare_computations(xs[1], r, xs)
        let comps = prepare_computations(&xs.data[1], &r, &xs, &no_parent);
    
        // And reflectance ← schlick(comps)
        let reflectance = schlick(&comps);
    
        // Then reflectance = 0.04
        assert!(crate::utils::equal(reflectance, 0.04));
    }
    
    #[test]
    fn the_schlick_approximation_with_small_angle_and_n2_greater_than_n1() {
        use crate::sphere::Sphere;
        use crate::ray::Ray;
        use crate::tuple::Tuple;
    
        // Given shape ← glass_sphere()
        let shape = Sphere::glass_sphere();
    
        // And r ← ray(point(0, 0.99, -2), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(0.0, 0.99, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And xs ← intersections(1.8589:shape)
        let xs = Intersections::new(vec![
            Intersection::new(1.8589, &shape),
        ]);
    
        // When comps ← prepare_computations(xs[0], r, xs)
        let comps = prepare_computations(&xs.data[0], &r, &xs, &no_parent);
    
        // And reflectance ← schlick(comps)
        let reflectance = schlick(&comps);
    
        // Then reflectance = 0.48873
        assert!(crate::utils::equal(reflectance, 0.48873));
    }

    #[test]
    fn prepare_computations_uses_parent_aware_normal_for_child_shapes() {
        use crate::group::Group;
        use crate::shape::shape_normal_at;
        use crate::transformation::{rotation_y, scaling, translation};
        use std::f64::consts::FRAC_PI_2;

        let mut g1 = Group::new();
        g1.set_transform(rotation_y(FRAC_PI_2));

        let mut g2 = Group::new();
        g2.set_transform(scaling(1.0, 2.0, 3.0));

        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));

        s.shape_data_mut().parent = Some(g2.id());
        g2.add_child(&s);

        g2.shape_data_mut().parent = Some(g1.id());
        g1.add_child(&g2);

        let resolve = |id: u64| -> Option<&dyn Shape> {
            if id == g1.id() {
                Some(&g1 as &dyn Shape)
            } else if id == g2.id() {
                Some(&g2 as &dyn Shape)
            } else if id == s.id() {
                Some(&s as &dyn Shape)
            } else {
                None
            }
        };

        let p = Tuple::point(1.7321, 1.1547, -5.5774);
        let r = Ray::new(p.clone(), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(0.0, &s);
        let xs = Intersections::new(vec![i.clone()]);
        let comps = prepare_computations(&i, &r, &xs, &resolve);
        let expected = shape_normal_at(&s, &resolve, &p);

        assert!(comps.normal_vector.is_equal(&expected));
        assert!(!comps.inside);
    }

    #[test]
    fn prepare_computations_flips_grouped_normal_when_hit_from_inside() {
        use crate::group::Group;
        use crate::shape::shape_normal_at;
        use crate::transformation::{rotation_y, scaling, translation};
        use std::f64::consts::FRAC_PI_2;

        let mut g1 = Group::new();
        g1.set_transform(rotation_y(FRAC_PI_2));

        let mut g2 = Group::new();
        g2.set_transform(scaling(1.0, 2.0, 3.0));

        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));

        s.shape_data_mut().parent = Some(g2.id());
        g2.add_child(&s);

        g2.shape_data_mut().parent = Some(g1.id());
        g1.add_child(&g2);

        let resolve = |id: u64| -> Option<&dyn Shape> {
            if id == g1.id() {
                Some(&g1 as &dyn Shape)
            } else if id == g2.id() {
                Some(&g2 as &dyn Shape)
            } else if id == s.id() {
                Some(&s as &dyn Shape)
            } else {
                None
            }
        };

        let p = Tuple::point(1.7321, 1.1547, -5.5774);
        let r = Ray::new(p.clone(), Tuple::vector(0.0, 0.0, -1.0));
        let i = Intersection::new(0.0, &s);
        let xs = Intersections::new(vec![i.clone()]);
        let comps = prepare_computations(&i, &r, &xs, &resolve);
        let expected = -&shape_normal_at(&s, &resolve, &p);

        assert!(comps.normal_vector.is_equal(&expected));
        assert!(comps.inside);
    }
}
