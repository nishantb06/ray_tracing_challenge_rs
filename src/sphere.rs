use std::sync::atomic::{AtomicU64, Ordering};

use crate::tuple::Tuple;
use crate::ray::Ray;
use crate::intersection::{Intersection, Intersections};

static NEXT_SPHERE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
#[allow(dead_code)]
pub struct Sphere {
    pub id: u64,
    pub center: Tuple,
    pub radius: f64,
}

#[allow(dead_code)]
impl Sphere {
    pub fn new() -> Self {
        Sphere {
            id: NEXT_SPHERE_ID.fetch_add(1, Ordering::Relaxed),
            center: Tuple::point(0.0, 0.0, 0.0),
            radius: 1.0,
        }
    }

    pub fn intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let sphere_to_ray = &ray.origin - &self.center;

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return Intersections::new(vec![]);
        }

        let sqrt_disc = discriminant.sqrt();
        let mut t1 = (-b - sqrt_disc) / (2.0 * a);
        let mut t2 = (-b + sqrt_disc) / (2.0 * a);

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        Intersections::new(vec![
            Intersection::new(t1, self),
            Intersection::new(t2, self),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sphere_has_unique_id() {
        let s1 = Sphere::new();
        let s2 = Sphere::new();
        assert_ne!(s1.id, s2.id);
    }

    #[test]
    fn a_ray_intersects_a_sphere_at_two_points() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 4.0));
        assert!(crate::utils::equal(xs.data[1].t, 6.0));
    }

    #[test]
    fn a_ray_intersects_a_sphere_at_a_tangent() {
        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 5.0));
        assert!(crate::utils::equal(xs.data[1].t, 5.0));
    }

    #[test]
    fn a_ray_misses_a_sphere() {
        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 0);
    }

    #[test]
    fn a_ray_originates_inside_a_sphere() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, -1.0));
        assert!(crate::utils::equal(xs.data[1].t, 1.0));
    }

    #[test]
    fn a_sphere_is_behind_a_ray() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, -6.0));
        assert!(crate::utils::equal(xs.data[1].t, -4.0));
    }

    #[test]
    fn intersections_are_returned_in_increasing_order() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(xs.data[0].t <= xs.data[1].t,
            "expected xs[0].t <= xs[1].t, got {} > {}", xs.data[0].t, xs.data[1].t);
    }

    #[test]
    fn intersect_sets_the_object_on_the_intersection() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.count(), 2);
        assert_eq!(xs.data[0].object.id, s.id);
        assert_eq!(xs.data[1].object.id, s.id);
    }
}
