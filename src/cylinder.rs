use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct Cylinder {
    pub data: ShapeData,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
}

impl Cylinder {
    pub fn new() -> Self {
        Cylinder {
            data: ShapeData::new(),
            minimum: f64::NEG_INFINITY,
            maximum: f64::INFINITY,
            closed: false,
        }
    }

    /// Helper function to intersect the caps of a closed cylinder.
    /// 'xs' is a mutable vector of intersections to add to.
    pub fn intersect_caps<'a>(&'a self, ray: &Ray, xs: &mut Vec<Intersection<'a>>) {
        // # caps only matter if the cylinder is closed, and might possibly be
        // # intersected by the ray.
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        // check for intersection with the lower end cap at y = self.minimum
        let t_lower = (self.minimum - ray.origin.y) / ray.direction.y;
        if check_cap(ray, t_lower) {
            xs.push(Intersection::new(t_lower, self));
        }

        // check for intersection with the upper end cap at y = self.maximum
        let t_upper = (self.maximum - ray.origin.y) / ray.direction.y;
        if check_cap(ray, t_upper) {
            xs.push(Intersection::new(t_upper, self));
        }
    }
}

/// Checks if the intersection at parameter t is within the unit radius of the cylinder caps.
/// Returns true if (x^2 + z^2) <= 1 for the computed point on the ray.
pub fn check_cap(ray: &Ray, t: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    (x * x + z * z) <= 1.0 + EPSILON
}


impl Shape for Cylinder {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
        let mut xs = vec![];

        // Ray is not parallel to the y axis — intersect the curved side
        if a.abs() >= EPSILON {
            let b = 2.0 * ray.origin.x * ray.direction.x
                + 2.0 * ray.origin.z * ray.direction.z;
            let c = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - 1.0;
            let disc = b * b - 4.0 * a * c;

            if disc >= 0.0 {
                let sqrt_disc = disc.sqrt();
                let mut t0 = (-b - sqrt_disc) / (2.0 * a);
                let mut t1 = (-b + sqrt_disc) / (2.0 * a);

                if t0 > t1 {
                    std::mem::swap(&mut t0, &mut t1);
                }

                let y0 = ray.origin.y + t0 * ray.direction.y;
                if self.minimum < y0 && y0 < self.maximum {
                    xs.push(Intersection::new(t0, self));
                }

                let y1 = ray.origin.y + t1 * ray.direction.y;
                if self.minimum < y1 && y1 < self.maximum {
                    xs.push(Intersection::new(t1, self));
                }
            }
        }

        // Always try caps for closed cylinders (handles ray parallel to axis and cap-only hits)
        self.intersect_caps(ray, &mut xs);
        Intersections::new(xs)
    }

    fn local_normal_at(&self, local_point: &Tuple) -> Tuple {
        let dist = local_point.x * local_point.x + local_point.z * local_point.z;

        if dist < 1.0 && local_point.y >= self.maximum - EPSILON {
            Tuple::vector(0.0, 1.0, 0.0)
        } else if dist < 1.0 && local_point.y <= self.minimum + EPSILON {
            Tuple::vector(0.0, -1.0, 0.0)
        } else {
            Tuple::vector(local_point.x, 0.0, local_point.z)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray::Ray;
    use crate::shape::Shape;
    use crate::tuple::Tuple;

    #[test]
    fn cube_is_a_shape() {
        fn assert_is_shape<T: Shape>(_: &T) {}
        let c = Cylinder::new();
        assert_is_shape(&c);
    }

    #[test]
    fn a_ray_misses_a_cylinder() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
        }
    
        let cases = [
            Case {
                origin: Tuple::point(1.0, 0.0, 0.0),
                direction: Tuple::vector(0.0, 1.0, 0.0),
            },
            Case {
                origin: Tuple::point(0.0, 0.0, 0.0),
                direction: Tuple::vector(0.0, 1.0, 0.0),
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(1.0, 1.0, 1.0),
            },
        ];
    
        let cyl = Cylinder::new();
    
        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.origin.clone(), direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(xs.count(), 0, "case {}: expected 0 intersections", i);
        }
    }

    #[test]
    fn a_ray_strikes_a_cylinder() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
            t0: f64,
            t1: f64,
        }
    
        let cases = [
            Case {
                origin: Tuple::point(1.0, 0.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                t0: 5.0,
                t1: 5.0,
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                t0: 4.0,
                t1: 6.0,
            },
            Case {
                origin: Tuple::point(0.5, 0.0, -5.0),
                direction: Tuple::vector(0.1, 1.0, 1.0),
                t0: 6.80798,
                t1: 7.08872,
            },
        ];
    
        let cyl = Cylinder::new();
    
        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.origin.clone(), direction);
            let xs = cyl.local_intersect(&r);
    
            assert_eq!(xs.count(), 2, "case {}: expected 2 intersections", i);
            assert!(
                crate::utils::equal(xs.data[0].t, tc.t0),
                "case {}: expected xs[0].t = {}, got {}",
                i,
                tc.t0,
                xs.data[0].t
            );
            assert!(
                crate::utils::equal(xs.data[1].t, tc.t1),
                "case {}: expected xs[1].t = {}, got {}",
                i,
                tc.t1,
                xs.data[1].t
            );
        }
    }

    #[test]
    fn normal_vector_on_a_cylinder() {
        struct Case {
            point: Tuple,
            normal: Tuple,
        }
    
        let cases = [
            Case {
                point: Tuple::point(1.0, 0.0, 0.0),
                normal: Tuple::vector(1.0, 0.0, 0.0),
            },
            Case {
                point: Tuple::point(0.0, 5.0, -1.0),
                normal: Tuple::vector(0.0, 0.0, -1.0),
            },
            Case {
                point: Tuple::point(0.0, -2.0, 1.0),
                normal: Tuple::vector(0.0, 0.0, 1.0),
            },
            Case {
                point: Tuple::point(-1.0, 1.0, 0.0),
                normal: Tuple::vector(-1.0, 0.0, 0.0),
            },
        ];
    
        let cyl = Cylinder::new();
    
        for (i, tc) in cases.iter().enumerate() {
            let n = cyl.local_normal_at(&tc.point);
            assert!(
                n.is_equal(&tc.normal),
                "case {}: expected normal {:?}, got {:?}",
                i,
                tc.normal,
                n
            );
        }
    }

    #[test]
    fn the_default_minimum_and_maximum_for_a_cylinder() {
        let cyl = Cylinder::new();
        assert!(
            cyl.minimum.is_infinite() && cyl.minimum.is_sign_negative(),
            "expected cyl.minimum = -infinity, got {}",
            cyl.minimum
        );
        assert!(
            cyl.maximum.is_infinite() && cyl.maximum.is_sign_positive(),
            "expected cyl.maximum = infinity, got {}",
            cyl.maximum
        );
    }

    #[test]
    fn intersecting_a_constrained_cylinder() {
        struct Case {
            point: Tuple,
            direction: Tuple,
            count: usize,
        }
    
        let cases = [
            Case {
                point: Tuple::point(0.0, 1.5, 0.0),
                direction: Tuple::vector(0.1, 1.0, 0.0),
                count: 0,
            },
            Case {
                point: Tuple::point(0.0, 3.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                count: 0,
            },
            Case {
                point: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                count: 0,
            },
            Case {
                point: Tuple::point(0.0, 2.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                count: 0,
            },
            Case {
                point: Tuple::point(0.0, 1.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                count: 0,
            },
            Case {
                point: Tuple::point(0.0, 1.5, -2.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                count: 2,
            },
        ];
    
        let mut cyl = Cylinder::new();
        cyl.minimum = 1.0;
        cyl.maximum = 2.0;
    
        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.point.clone(), direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(
                xs.count(),
                tc.count,
                "case {}: expected {} intersections, got {}",
                i + 1,
                tc.count,
                xs.count()
            );
        }
    }

    #[test]
    fn the_default_closed_value_for_a_cylinder() {
        let cyl = Cylinder::new();
        assert!(!cyl.closed, "expected cyl.closed = false, got {}", cyl.closed);
    }

    #[test]
    fn intersecting_the_caps_of_a_closed_cylinder() {
        struct Case {
            point: Tuple,
            direction: Tuple,
            count: usize,
        }
    
        let cases = [
            Case {
                point: Tuple::point(0.0, 3.0, 0.0),
                direction: Tuple::vector(0.0, -1.0, 0.0),
                count: 2,
            },
            Case {
                point: Tuple::point(0.0, 3.0, -2.0),
                direction: Tuple::vector(0.0, -1.0, 2.0),
                count: 2,
            },
            Case {
                point: Tuple::point(0.0, 4.0, -2.0),
                direction: Tuple::vector(0.0, -1.0, 1.0),
                count: 2,
            },
            Case {
                point: Tuple::point(0.0, 0.0, -2.0),
                direction: Tuple::vector(0.0, 1.0, 2.0),
                count: 2,
            },
            Case {
                point: Tuple::point(0.0, -1.0, -2.0),
                direction: Tuple::vector(0.0, 1.0, 1.0),
                count: 2,
            },
        ];
    
        let mut cyl = Cylinder::new();
        cyl.minimum = 1.0;
        cyl.maximum = 2.0;
        cyl.closed = true;
    
        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.point.clone(), direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(
                xs.count(),
                tc.count,
                "case {}: expected {} intersections, got {}",
                i + 1,
                tc.count,
                xs.count()
            );
        }
    }

    #[test]
    fn the_normal_vector_on_a_cylinders_end_caps() {
        struct Case {
            point: Tuple,
            normal: Tuple,
        }
    
        let cases = [
            Case {
                point: Tuple::point(0.0, 1.0, 0.0),
                normal: Tuple::vector(0.0, -1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.5, 1.0, 0.0),
                normal: Tuple::vector(0.0, -1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.0, 1.0, 0.5),
                normal: Tuple::vector(0.0, -1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.0, 2.0, 0.0),
                normal: Tuple::vector(0.0, 1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.5, 2.0, 0.0),
                normal: Tuple::vector(0.0, 1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.0, 2.0, 0.5),
                normal: Tuple::vector(0.0, 1.0, 0.0),
            },
        ];
    
        let mut cyl = Cylinder::new();
        cyl.minimum = 1.0;
        cyl.maximum = 2.0;
        cyl.closed = true;
    
        for (i, tc) in cases.iter().enumerate() {
            let n = cyl.local_normal_at(&tc.point);
            assert!(
                n.is_equal(&tc.normal),
                "case {}: expected normal {:?}, got {:?}",
                i,
                tc.normal,
                n
            );
        }
    }
}