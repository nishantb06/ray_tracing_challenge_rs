use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
pub struct Cone {
    pub data: ShapeData,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
}

impl Cone {
    pub fn new() -> Self {
        Cone {
            data: ShapeData::new(),
            minimum: f64::NEG_INFINITY,
            maximum: f64::INFINITY,
            closed: false,
        }
    }

    /// Intersect the end caps of a closed cone.
    /// The cap at height y has radius |y|, so the check radius is y (passed in).
    pub fn intersect_caps<'a>(&'a self, ray: &Ray, xs: &mut Vec<Intersection<'a>>) {
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return;
        }

        let t_lower = (self.minimum - ray.origin.y) / ray.direction.y;
        if check_cap(ray, t_lower, self.minimum) {
            xs.push(Intersection::new(t_lower, self));
        }

        let t_upper = (self.maximum - ray.origin.y) / ray.direction.y;
        if check_cap(ray, t_upper, self.maximum) {
            xs.push(Intersection::new(t_upper, self));
        }
    }
}

/// Checks whether the ray at parameter `t` lands within a cone cap of radius `|y|`.
pub fn check_cap(ray: &Ray, t: f64, y: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    (x * x + z * z) <= y * y + EPSILON
}

impl Shape for Cone {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let dx = ray.direction.x;
        let dy = ray.direction.y;
        let dz = ray.direction.z;
        let ox = ray.origin.x;
        let oy = ray.origin.y;
        let oz = ray.origin.z;

        let a = dx * dx - dy * dy + dz * dz;
        let b = 2.0 * ox * dx - 2.0 * oy * dy + 2.0 * oz * dz;
        let c = ox * ox - oy * oy + oz * oz;

        let mut xs: Vec<Intersection> = vec![];

        if a.abs() < EPSILON {
            // Ray is parallel to one nappe of the cone
            if b.abs() >= EPSILON {
                // Single intersection
                let t = -c / (2.0 * b);
                let y = oy + t * dy;
                if self.minimum < y && y < self.maximum {
                    xs.push(Intersection::new(t, self));
                }
            }
            // If both a and b are ~0 the ray misses entirely — fall through to caps
        } else {
            let disc = b * b - 4.0 * a * c;

            if disc >= 0.0 {
                let sqrt_disc = disc.sqrt();
                let mut t0 = (-b - sqrt_disc) / (2.0 * a);
                let mut t1 = (-b + sqrt_disc) / (2.0 * a);

                if t0 > t1 {
                    std::mem::swap(&mut t0, &mut t1);
                }

                let y0 = oy + t0 * dy;
                if self.minimum < y0 && y0 < self.maximum {
                    xs.push(Intersection::new(t0, self));
                }

                let y1 = oy + t1 * dy;
                if self.minimum < y1 && y1 < self.maximum {
                    xs.push(Intersection::new(t1, self));
                }
            }
        }

        self.intersect_caps(ray, &mut xs);
        Intersections::new(xs)
    }

    fn local_normal_at(&self, local_point: &Tuple, _hit: Option<&Intersection>) -> Tuple {
        let dist = local_point.x * local_point.x + local_point.z * local_point.z;

        // Check if the point is on a cap (only when closed)
        if self.closed {
            if dist <= self.maximum * self.maximum + EPSILON
                && local_point.y >= self.maximum - EPSILON
            {
                return Tuple::vector(0.0, 1.0, 0.0);
            }
            if dist <= self.minimum * self.minimum + EPSILON
                && local_point.y <= self.minimum + EPSILON
            {
                return Tuple::vector(0.0, -1.0, 0.0);
            }
        }

        // Curved surface normal: y = ±sqrt(x²+z²), negated when point.y > 0
        let mut y = dist.sqrt();
        if local_point.y > 0.0 {
            y = -y;
        }
        Tuple::vector(local_point.x, y, local_point.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray::Ray;
    use crate::shape::Shape;
    use crate::tuple::Tuple;

    #[test]
    fn cone_is_a_shape() {
        fn assert_is_shape<T: Shape>(_: &T) {}
        let c = Cone::new();
        assert_is_shape(&c);
    }

    #[test]
    fn intersecting_a_cone_with_a_ray() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
            t0: f64,
            t1: f64,
        }

        let cases = [
            Case {
                origin: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(0.0, 0.0, 1.0),
                t0: 5.0,
                t1: 5.0,
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(1.0, 1.0, 1.0),
                t0: 8.66025,
                t1: 8.66025,
            },
            Case {
                origin: Tuple::point(1.0, 1.0, -5.0),
                direction: Tuple::vector(-0.5, -1.0, 1.0),
                t0: 4.55006,
                t1: 49.44994,
            },
        ];

        let shape = Cone::new();

        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.origin.clone(), direction);
            let xs = shape.local_intersect(&r);

            assert_eq!(xs.count(), 2, "case {}: expected 2 intersections, got {}", i, xs.count());
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
    fn intersecting_a_cone_with_a_ray_parallel_to_one_half() {
        let shape = Cone::new();
        let direction = Tuple::vector(0.0, 1.0, 1.0).normalize();
        let r = Ray::new(Tuple::point(0.0, 0.0, -1.0), direction);
        let xs = shape.local_intersect(&r);

        assert_eq!(xs.count(), 1, "expected 1 intersection, got {}", xs.count());
        assert!(
            crate::utils::equal(xs.data[0].t, 0.35355),
            "expected xs[0].t = 0.35355, got {}",
            xs.data[0].t
        );
    }

    #[test]
    fn intersecting_a_cones_end_caps() {
        struct Case {
            origin: Tuple,
            direction: Tuple,
            count: usize,
        }

        let cases = [
            Case {
                origin: Tuple::point(0.0, 0.0, -5.0),
                direction: Tuple::vector(0.0, 1.0, 0.0),
                count: 0,
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -0.25),
                direction: Tuple::vector(0.0, 1.0, 1.0),
                count: 2,
            },
            Case {
                origin: Tuple::point(0.0, 0.0, -0.25),
                direction: Tuple::vector(0.0, 1.0, 0.0),
                count: 4,
            },
        ];

        let mut shape = Cone::new();
        shape.minimum = -0.5;
        shape.maximum = 0.5;
        shape.closed = true;

        for (i, tc) in cases.iter().enumerate() {
            let direction = tc.direction.normalize();
            let r = Ray::new(tc.origin.clone(), direction);
            let xs = shape.local_intersect(&r);
            assert_eq!(
                xs.count(),
                tc.count,
                "case {}: expected {} intersections, got {}",
                i,
                tc.count,
                xs.count()
            );
        }
    }

    #[test]
    fn computing_the_normal_vector_on_a_cone() {
        struct Case {
            point: Tuple,
            normal: Tuple,
        }

        let cases = [
            Case {
                point: Tuple::point(0.0, 0.0, 0.0),
                normal: Tuple::vector(0.0, 0.0, 0.0),
            },
            Case {
                point: Tuple::point(1.0, 1.0, 1.0),
                normal: Tuple::vector(1.0, -(2.0_f64.sqrt()), 1.0),
            },
            Case {
                point: Tuple::point(-1.0, -1.0, 0.0),
                normal: Tuple::vector(-1.0, 1.0, 0.0),
            },
        ];

        let shape = Cone::new();

        for (i, tc) in cases.iter().enumerate() {
            let n = shape.local_normal_at(&tc.point,None);
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
