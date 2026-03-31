use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct Cube {
    pub data: ShapeData,
}

impl Cube {
    pub fn new() -> Self {
        Cube {
            data: ShapeData::new(),
        }
    }
}

// Returns (tmin, tmax) for a slab from -1 to +1 on one axis.
pub fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
    let tmin_numerator = -1.0 - origin;
    let tmax_numerator = 1.0 - origin;

    let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
        (tmin_numerator / direction, tmax_numerator / direction)
    } else {
        // Ray is (effectively) parallel to the planes; use infinities.
        (tmin_numerator * f64::INFINITY, tmax_numerator * f64::INFINITY)
    };

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    (tmin, tmax)
}

impl Shape for Cube {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let (xtmin, xtmax) = check_axis(ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) = check_axis(ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) = check_axis(ray.origin.z, ray.direction.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        if tmin > tmax {
            return Intersections::new(vec![]);
        }

        Intersections::new(vec![Intersection::new(tmin, self), Intersection::new(tmax, self)])
    }

    fn local_normal_at(&self, local_point: &Tuple,_hit: Option<&Intersection>) -> Tuple {
        let maxc = local_point.x.abs().max(local_point.y.abs()).max(local_point.z.abs());

        if maxc == local_point.x.abs() {
            Tuple::vector(local_point.x.signum(), 0.0, 0.0)
        } else if maxc == local_point.y.abs() {
            Tuple::vector(0.0, local_point.y.signum(), 0.0)
        } else {
            Tuple::vector(0.0, 0.0, local_point.z.signum())
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
        let c = Cube::new();
        assert_is_shape(&c);
    }

    #[test]
    fn a_ray_intersects_a_cube() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
            t1: f64,
            t2: f64,
        }

        let cases = [
            Case {
                origin: Tuple::point(5.0, 0.5, 0.0),
                direction: Tuple::vector(-1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(-5.0, 0.5, 0.0),
                direction: Tuple::vector(1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(0.5, 5.0, 0.0),
                direction: Tuple::vector(0.0, -1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(0.5, -5.0, 0.0),
                direction: Tuple::vector(0.0, 1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(0.5, 0.0, 5.0),
                direction: Tuple::vector(0.0, 0.0, -1.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(0.5, 0.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                t1: 4.0,
                t2: 6.0,
            },
            Case {
                origin: Tuple::point(0.0, 0.5, 0.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                t1: -1.0,
                t2: 1.0,
            },
        ];

        let c = Cube::new();

        for (i, tc) in cases.iter().enumerate() {
            let r = Ray::new(tc.origin.clone(), tc.direction.clone());
            let xs = c.local_intersect(&r);

            assert_eq!(xs.count(), 2, "case {}: expected 2 intersections", i);
            assert!(
                crate::utils::equal(xs.data[0].t, tc.t1),
                "case {}: expected xs[0].t = {}, got {}",
                i,
                tc.t1,
                xs.data[0].t
            );
            assert!(
                crate::utils::equal(xs.data[1].t, tc.t2),
                "case {}: expected xs[1].t = {}, got {}",
                i,
                tc.t2,
                xs.data[1].t
            );
        }
    }

    #[test]
    fn check_axis_handles_parallel_ray_by_returning_infinities() {
        // direction "effectively 0" should trigger the infinity path
        // (assuming check_axis uses abs(direction) >= EPSILON like the book)
        let (tmin, tmax) = check_axis(0.0, 0.0);

        assert!(tmin.is_infinite() && tmin.is_sign_negative(), "tmin = {}", tmin);
        assert!(tmax.is_infinite() && tmax.is_sign_positive(), "tmax = {}", tmax);
    }

    #[test]
    fn a_ray_misses_a_cube() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
        }
    
        let cases = [
            Case {
                origin: Tuple::point(-2.0, 0.0, 0.0),
                direction: Tuple::vector(0.2673, 0.5345, 0.8018),
            },
            Case {
                origin: Tuple::point(0.0, -2.0, 0.0),
                direction: Tuple::vector(0.8018, 0.2673, 0.5345),
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -2.0),
                direction: Tuple::vector(0.5345, 0.8018, 0.2673),
            },
            Case {
                origin: Tuple::point(2.0, 0.0, 2.0),
                direction: Tuple::vector(0.0, 0.0, -1.0),
            },
            Case {
                origin: Tuple::point(0.0, 2.0, 2.0),
                direction: Tuple::vector(0.0, -1.0, 0.0),
            },
            Case {
                origin: Tuple::point(2.0, 2.0, 0.0),
                direction: Tuple::vector(-1.0, 0.0, 0.0),
            },
        ];
    
        let c = Cube::new();
    
        for (i, tc) in cases.iter().enumerate() {
            let r = Ray::new(tc.origin.clone(), tc.direction.clone());
            let xs = c.local_intersect(&r);
            assert_eq!(xs.count(), 0, "case {}: expected 0 intersections", i);
        }
    }

    #[test]
    fn the_normal_on_the_surface_of_a_cube() {
        struct Case {
            point: Tuple,
            normal: Tuple,
        }
    
        let cases = [
            Case {
                point: Tuple::point(1.0, 0.5, -0.8),
                normal: Tuple::vector(1.0, 0.0, 0.0),
            },
            Case {
                point: Tuple::point(-1.0, -0.2, 0.9),
                normal: Tuple::vector(-1.0, 0.0, 0.0),
            },
            Case {
                point: Tuple::point(-0.4, 1.0, -0.1),
                normal: Tuple::vector(0.0, 1.0, 0.0),
            },
            Case {
                point: Tuple::point(0.3, -1.0, -0.7),
                normal: Tuple::vector(0.0, -1.0, 0.0),
            },
            Case {
                point: Tuple::point(-0.6, 0.3, 1.0),
                normal: Tuple::vector(0.0, 0.0, 1.0),
            },
            Case {
                point: Tuple::point(0.4, 0.4, -1.0),
                normal: Tuple::vector(0.0, 0.0, -1.0),
            },
            Case {
                point: Tuple::point(1.0, 1.0, 1.0),
                normal: Tuple::vector(1.0, 0.0, 0.0),
            },
            Case {
                point: Tuple::point(-1.0, -1.0, -1.0),
                normal: Tuple::vector(-1.0, 0.0, 0.0),
            },
        ];
    
        let c = Cube::new();
    
        for (i, tc) in cases.iter().enumerate() {
            let n = c.local_normal_at(&tc.point,None);
            assert!(
                n.is_equal(&tc.normal),
                "case {}: expected {:?}, got {:?}",
                i,
                tc.normal,
                n
            );
        }
    }
}