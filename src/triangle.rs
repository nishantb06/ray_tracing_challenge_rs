use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct Triangle {
    pub data: ShapeData,
    pub p1: Tuple,
    pub p2: Tuple,
    pub p3: Tuple,
    pub e1: Tuple,
    pub e2: Tuple,
    pub normal: Tuple,
}

impl Triangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple) -> Self {
        let e1 = &p2 - &p1;
        let e2 = &p3 - &p1;
        let normal = e2.cross(&e1).normalize();
        Triangle {
            data: ShapeData::new(),
            p1,
            p2,
            p3,
            e1,
            e2,
            normal,
        }
    }
}

impl Shape for Triangle {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let dir_cross_e2 = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&dir_cross_e2);
        if det.abs() < EPSILON {
            return Intersections::new(vec![]);
        }
        let f = 1.0 / det;
        let p1_to_origin = &ray.origin - &self.p1;
        let u = f * p1_to_origin.dot(&dir_cross_e2);
        if u < 0.0 || u > 1.0 {
            return Intersections::new(vec![]);
        }
        let origin_cross_e1 = p1_to_origin.cross(&self.e1);
        let v = f * ray.direction.dot(&origin_cross_e1);
        if v < 0.0 || (u + v) > 1.0 {
            return Intersections::new(vec![]);
        }
        let t_hit = f * self.e2.dot(&origin_cross_e1);
        Intersections::new(vec![Intersection::intersection_with_uv(t_hit, self, u, v)])
    }

    fn local_normal_at(&self, _local_point: &Tuple,_hit: Option<&Intersection>) -> Tuple {
        return self.normal.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuple::Tuple;

    #[test]
    fn constructing_a_triangle() {
        let p1 = Tuple::point(0.0, 1.0, 0.0);
        let p2 = Tuple::point(-1.0, 0.0, 0.0);
        let p3 = Tuple::point(1.0, 0.0, 0.0);

        let t = Triangle::new(p1.clone(), p2.clone(), p3.clone());

        // Check vertices
        assert_eq!(t.p1 ,p1);
        assert_eq!(t.p2, p2);
        assert_eq!(t.p3, p3);

        // Check edge vectors
        assert_eq!(t.e1, Tuple::vector(-1.0, -1.0, 0.0));
        assert_eq!(t.e2, Tuple::vector(1.0, -1.0, 0.0));

        // Check normal
        assert_eq!(t.normal, Tuple::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn finding_the_normal_on_a_triangle() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
    
        let n1 = t.local_normal_at(&Tuple::point(0.0, 0.5, 0.0),None);
        let n2 = t.local_normal_at(&Tuple::point(-0.5, 0.75, 0.0),None);
        let n3 = t.local_normal_at(&Tuple::point(0.5, 0.25, 0.0),None);
    
        assert_eq!(n1, t.normal);
        assert_eq!(n2, t.normal);
        assert_eq!(n3, t.normal);
    }
    #[test]
    fn intersecting_a_ray_parallel_to_the_triangle() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, -1.0, -2.0),
            Tuple::vector(0.0, 1.0, 0.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.count(), 0);
    }
    
    #[test]
    fn a_ray_misses_the_p1_p3_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(1.0, 1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.count(), 0);
    }
    #[test]
    fn a_ray_misses_the_p1_p2_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(-1.0, 1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        assert_eq!(t.local_intersect(&r).count(), 0);
    }
    
    #[test]
    fn a_ray_misses_the_p2_p3_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, -1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        assert_eq!(t.local_intersect(&r).count(), 0);
    }
    
    #[test]
    fn a_ray_strikes_a_triangle() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, 0.5, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.count(), 1);
        assert!(crate::utils::equal(xs.data[0].t, 2.0));
    }
}