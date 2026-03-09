use crate::tuple::Tuple;
use crate::utils::equal;
use crate::matrix::Matrix;
use crate::transformation::{translation, scaling};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

#[allow(dead_code)]
impl Ray {
    pub fn new(origin: Tuple, direction: Tuple) -> Self {
        assert!(
            equal(origin.w, 1.0),
            "Ray origin must be a point (w == 1), got w = {}",
            origin.w
        );
        assert!(
            equal(direction.w, 0.0),
            "Ray direction must be a vector (w == 0), got w = {}",
            direction.w
        );
        Ray {
            origin,
            direction,
        }
    }

    pub fn position(&self, t: f64) -> Tuple {
        &self.origin + &(&self.direction * t)
    }

    pub fn transform(&self, transformation: &Matrix) -> Self {
        Ray {
            origin: transformation * &self.origin,
            direction: transformation * &self.direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_and_querying_a_ray() {
        let origin = Tuple::point(1.0, 2.0, 3.0);
        let direction = Tuple::vector(4.0, 5.0, 6.0);
        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(4.0, 5.0, 6.0));
        assert!(r.origin.is_equal(&origin));
        assert!(r.direction.is_equal(&direction));
    }

    #[test]
    fn computing_a_point_from_a_distance() {
        let r = Ray::new(Tuple::point(2.0, 3.0, 4.0), Tuple::vector(1.0, 0.0, 0.0));
        assert!(r.position(0.0).is_equal(&Tuple::point(2.0, 3.0, 4.0)));
        assert!(r.position(1.0).is_equal(&Tuple::point(3.0, 3.0, 4.0)));
        assert!(r.position(-1.0).is_equal(&Tuple::point(1.0, 3.0, 4.0)));
        assert!(r.position(2.5).is_equal(&Tuple::point(4.5, 3.0, 4.0)));
    }

    #[test]
    fn translating_a_ray() {
        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let m = translation(3.0, 4.0, 5.0);
        let r2 = r.transform(&m);
        assert!(r2.origin.is_equal(&Tuple::point(4.0, 6.0, 8.0)));
        assert!(r2.direction.is_equal(&Tuple::vector(0.0, 1.0, 0.0)));
    }

    #[test]
    fn scaling_a_ray() {
        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let m = scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(&m);
        assert!(r2.origin.is_equal(&Tuple::point(2.0, 6.0, 12.0)));
        assert!(r2.direction.is_equal(&Tuple::vector(0.0, 3.0, 0.0)));
    }
}

