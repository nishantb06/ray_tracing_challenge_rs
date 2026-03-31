use crate::ray::Ray;
use crate::tuple::Tuple;
use crate::intersection::{Intersection, Intersections};
use crate::shape::{ShapeData, Shape};
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct Plane {
    pub data: ShapeData,
}

impl Plane {
    pub fn new() -> Self {
        Plane {
            data: ShapeData::new(),
        }
    }
}

impl Shape for Plane {
    fn shape_data(&self) -> &ShapeData { &self.data }
    fn shape_data_mut(&mut self) -> &mut ShapeData { &mut self.data }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        if ray.direction.y.abs() < EPSILON {
            return Intersections::new(vec![]);
        }
        let t = -ray.origin.y / ray.direction.y;
        Intersections::new(vec![Intersection::new(t, self)])
    }

    fn local_normal_at(&self, _local_point: &Tuple,_hit: Option<&Intersection>) -> Tuple {
        Tuple::vector(0.0, 1.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray::Ray;
    use crate::tuple::Tuple;
    use crate::shape::Shape;

    fn plane() -> Plane {
        Plane::new()
    }

    #[test]
    fn plane_is_a_shape() {
        fn assert_is_shape<T: Shape>(_: &T) {}
        let p = plane();
        assert_is_shape(&p);
    }

    #[test]
    fn the_normal_of_a_plane_is_constant_everywhere() {
        let p = plane();
        let n1 = p.local_normal_at(&Tuple::point(0.0, 0.0, 0.0),None);
        let n2 = p.local_normal_at(&Tuple::point(10.0, 0.0, -10.0),None);
        let n3 = p.local_normal_at(&Tuple::point(-5.0, 0.0, 150.0),None);
        assert!(n1.is_equal(&Tuple::vector(0.0, 1.0, 0.0)));
        assert!(n2.is_equal(&Tuple::vector(0.0, 1.0, 0.0)));
        assert!(n3.is_equal(&Tuple::vector(0.0, 1.0, 0.0)));
    }

    #[test]
    fn intersect_with_a_ray_parallel_to_the_plane() {
        let p = plane();
        let r = Ray::new(Tuple::point(0.0, 10.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.count(), 0);
    }

    #[test]
    fn intersect_with_a_coplanar_ray() {
        let p = plane();
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.count(), 0);
    }

    #[test]
    fn a_ray_intersecting_a_plane_from_above() {
        let p = plane();
        let r = Ray::new(Tuple::point(0.0, 1.0, 0.0), Tuple::vector(0.0, -1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.count(), 1);
        assert!(crate::utils::equal(xs.data[0].t, 1.0));
        assert_eq!(xs.data[0].object.id(), p.data.id);
    }

    #[test]
    fn a_ray_intersecting_a_plane_from_below() {
        let p = plane();
        let r = Ray::new(Tuple::point(0.0, -1.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.count(), 1);
        assert!(crate::utils::equal(xs.data[0].t, 1.0));
        assert_eq!(xs.data[0].object.id(), p.data.id);
    }
}