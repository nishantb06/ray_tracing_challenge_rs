use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct SmoothTriangle {
    pub data: ShapeData,

    pub p1: Tuple,
    pub p2: Tuple,
    pub p3: Tuple,

    pub n1: Tuple,
    pub n2: Tuple,
    pub n3: Tuple,

    pub e1: Tuple,
    pub e2: Tuple,
}


impl SmoothTriangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple, n1: Tuple, n2: Tuple, n3: Tuple ) -> Self {
        let e1 = &p2 - &p1;
        let e2 = &p3 - &p1;
        SmoothTriangle {
            data: ShapeData::new(),
            p1,
            p2,
            p3,
            n1,
            n2,
            n3,
            e1,
            e2,
        }
    }

    pub fn normal_at_with_hit(&self, _local_point: &Tuple, hit: &Intersection) -> Tuple {
        let u = hit.u.expect("SmoothTriangle normal requires u");
        let v = hit.v.expect("SmoothTriangle normal requires v");
    
        let t2 = &self.n2 * u;
        let t3 = &self.n3 * v;
        let t1 = &self.n1 * (1.0 - u - v);
    
        let n = &t2 + &(&t3 + &t1);
        n.normalize()
    }
}

impl Shape for SmoothTriangle {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        // all of this will be replaced hopefully
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

    fn local_normal_at(&self, _local_point: &Tuple, hit: Option<&Intersection>) -> Tuple {
        let hit = hit.expect("SmoothTriangle local_normal_at requires hit with u/v");
        let u = hit.u.expect("SmoothTriangle normal requires u");
        let v = hit.v.expect("SmoothTriangle normal requires v");
        let t2 = &self.n2 * u;
        let t3 = &self.n3 * v;
        let t1 = &self.n1 * (1.0 - u - v);
        let n = &t2 + &(&t3 + &t1);
        n.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intersection::prepare_computations;
    use crate::tuple::Tuple;
    use crate::utils::equal;

    // By default, Rust warns about unused (dead) code that is not used outside the module. 
    // The `test_triangle` helper is only used by test functions, which are only compiled and run with `cargo test`. 
    // So during a normal build (not `cargo test`), this function appears unused to the compiler.
    //
    // Solution: mark the function with `#[allow(dead_code)]` to suppress the warning.
     // Local helper: smooth-triangle tests don't care about parents.
    fn no_parent<'a>(_id: u64) -> Option<&'a dyn Shape> {
        None
    }
    
    #[allow(dead_code)]
    fn test_triangle() -> SmoothTriangle {
        let p1 = Tuple::point(0.0, 1.0, 0.0);
        let p2 = Tuple::point(-1.0, 0.0, 0.0);
        let p3 = Tuple::point(1.0, 0.0, 0.0);
        let n1 = Tuple::vector(0.0, 1.0, 0.0);
        let n2 = Tuple::vector(-1.0, 0.0, 0.0);
        let n3 = Tuple::vector(1.0, 0.0, 0.0);

        SmoothTriangle::new(p1, p2, p3, n1, n2, n3)
    }

    #[test]
    fn constructing_a_smooth_triangle() {
        let p1 = Tuple::point(0.0, 1.0, 0.0);
        let p2 = Tuple::point(-1.0, 0.0, 0.0);
        let p3 = Tuple::point(1.0, 0.0, 0.0);

        let n1 = Tuple::vector(0.0, 1.0, 0.0);
        let n2 = Tuple::vector(-1.0, 0.0, 0.0);
        let n3 = Tuple::vector(1.0, 0.0, 0.0);

        // let tri = SmoothTriangle::new(p1.clone(), p2.clone(), p3.clone(), n1.clone(), n2.clone(), n3.clone());
        let tri = test_triangle();

        assert_eq!(tri.p1, p1);
        assert_eq!(tri.p2, p2);
        assert_eq!(tri.p3, p3);

        assert_eq!(tri.n1, n1);
        assert_eq!(tri.n2, n2);
        assert_eq!(tri.n3, n3);
    }

    #[test]
    fn an_intersection_with_a_smooth_triangle_stores_u_and_v() {
        let tri = test_triangle(); // uses your helper that builds a SmoothTriangle
    
        let r = Ray::new(
            Tuple::point(-0.2, 0.3, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        let xs = tri.local_intersect(&r);
        let i0 = &xs.data[0];
    
        assert!(equal(i0.u.unwrap(), 0.45));
        assert!(equal(i0.v.unwrap(), 0.25));
    }
    
    #[test]
    fn a_smooth_triangle_uses_u_and_v_to_interpolate_the_normal() {
        use crate::intersection::Intersection;
    
        let tri = test_triangle();
    
        // Given i ← intersection_with_uv(1, tri, 0.45, 0.25)
        let i = Intersection::intersection_with_uv(1.0, &tri, 0.45, 0.25);
    
        // When n ← normal_at(tri, point(0, 0, 0), i)
        // Here we work in object space, so pass a local point (0,0,0)
        let n = tri.local_normal_at(&Tuple::point(0.0, 0.0, 0.0), Some(&i));
    
        // Then n = vector(-0.5547, 0.83205, 0)
        let expected = Tuple::vector(-0.5547, 0.83205, 0.0);
        assert!(n.is_equal(&expected));
    }

    #[test]
    fn preparing_the_normal_on_a_smooth_triangle() {
    
        let tri = test_triangle();
    
        // When i ← intersection_with_uv(1, tri, 0.45, 0.25)
        let i = Intersection::intersection_with_uv(1.0, &tri, 0.45, 0.25);
    
        // And r ← ray(point(-0.2, 0.3, -2), vector(0, 0, 1))
        let r = Ray::new(
            Tuple::point(-0.2, 0.3, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
    
        // And xs ← intersections(i)
        let xs = Intersections::new(vec![i.clone()]);
    
        // And comps ← prepare_computations(i, r, xs)
        let comps = prepare_computations(&i, &r, &xs, &no_parent);
    
        // Then comps.normalv = vector(-0.5547, 0.83205, 0)
        let expected = Tuple::vector(-0.5547, 0.83205, 0.0);
        assert!(comps.normal_vector.is_equal(&expected));
    }
}