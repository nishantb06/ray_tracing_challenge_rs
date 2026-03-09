use crate::tuple::Tuple;
use crate::utils::equal;

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
}

