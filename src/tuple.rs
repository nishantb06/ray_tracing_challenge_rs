use std::ops::{Add,Sub,Neg,Div,Mul};
use crate::utils::{equal};

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)] // suppresses the warnings
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[allow(dead_code)]
impl Tuple {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn point(x: f64, y:f64, z: f64) -> Self {
        Self { x, y, z, w : 1.0 }
    }

    pub fn vector(x: f64, y:f64, z: f64) -> Self {
        Self { x, y, z, w : 0.0 }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        return equal(self.x, other.x) 
            && equal(self.y, other.y)
            && equal(self.z, other.z)
            && equal(self.w, other.w)
    }

    pub fn magnitude(&self) -> f64 {
        debug_assert!(self.w == 0.0, "magnitude is typically for vectors (w=0)");
        self.x.hypot(self.y).hypot(self.z)
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        debug_assert!(self.w == 0.0, "normalize should only be called on vectors (w = 0.0)");
        debug_assert!(mag >= crate::utils::EPSILON, "Cannot normalize zero vector");
        assert!(mag >= crate::utils::EPSILON, "Cannot normalize zero vector");
        Tuple {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
            w: 0.0,
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn cross(&self, other: &Self) -> Self {
        Tuple {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
            w: 0.0,
        }
    }
}

impl Add for &Tuple {
    type Output = Tuple;

    fn add(self, rhs: Self) -> Self::Output {
        if self.w == 1.0 && rhs.w == 1.0 {
            panic!("Cannot add two points");
        }
        Tuple {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Sub for &Tuple {
    type Output = Tuple;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.w == 0.0 && rhs.w == 1.0 {
            panic!("Cannot subtract a point from a vector");
        }
        Tuple {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Neg for &Tuple {
    type Output = Tuple;

    fn neg(self) -> Self::Output {
        Tuple {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

// Tuple * scalar
impl Mul<f64> for &Tuple {
    type Output = Tuple;

    fn mul(self, rhs: f64) -> Self::Output {
        Tuple {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

// scalar * Tuple (so you can write 2.0 * t)
impl Mul<&Tuple> for f64 {
    type Output = Tuple;

    fn mul(self, rhs: &Tuple) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for &Tuple {
    type Output = Tuple;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Attempted to divide tuple by zero");
        }
        Tuple {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_creates_tuple_with_w_zero() {
        let v = Tuple::vector(1.0, 2.0, 3.0);
        let t = Tuple::new(1.0, 2.0, 3.0, 0.0);
        // assert_eq!(v, t);
        assert!(v.is_equal(&t), "Tuple::vector did not create a tuple with w = 0.0 as expected: {:?} vs {:?}", v, t);
    }

    #[test]
    fn point_creates_tuple_with_w_one() {
        let p = Tuple::point(1.0, 2.0, 3.0);
        let t = Tuple::new(1.0, 2.0, 3.0, 1.0);
        // assert_eq!(p, t);
        assert!(p.is_equal(&t), "Tuple::point did not create a tuple with w = 1.0 as expected: {:?} vs {:?}", p, t);
    }

    #[test]
    fn adding_two_tuples() {
        let a1 = Tuple::new(3.0, -2.0, 5.0, 1.0);
        let a2 = Tuple::new(-2.0, 3.0, 1.0, 0.0);
        let expected = Tuple::new(1.0, 1.0, 6.0, 1.0);
        assert!(a1.add(&a2).is_equal(&expected)); // reference coercion happens here. a1 -> (&a1)
        // with the operator:
        assert!((&a1 + &a2).is_equal(&expected));
    }
    #[test]
    #[should_panic(expected = "Cannot add two points")]
    fn adding_two_points_panics() {
        let p1 = Tuple::point(1.0, 2.0, 3.0);
        let p2 = Tuple::point(4.0, 5.0, 6.0);
        let _ = &p1 + &p2;
    }

    #[test]
    fn subtracting_two_points() {
        let p1 = Tuple::point(3.0, 2.0, 1.0);
        let p2 = Tuple::point(5.0, 6.0, 7.0);
        let expected = Tuple::vector(-2.0, -4.0, -6.0);
        assert!((&p1 - &p2).is_equal(&expected));
        assert!(p1.sub(&p2).is_equal(&expected));
    }

    #[test]
    fn subtracting_two_vectors() {
        let v1 = Tuple::vector(3.0, 2.0, 1.0);
        let v2 = Tuple::vector(5.0, 6.0, 7.0);
        let expected = Tuple::vector(-2.0, -4.0, -6.0);
        assert!((&v1 - &v2).is_equal(&expected))
    }

    #[test]
    fn subtracting_vector_from_point() {
        let p = Tuple::point(3.0, 2.0, 1.0);
        let v = Tuple::vector(5.0, 6.0, 7.0);
        let expected = Tuple::point(-2.0, -4.0, -6.0);
        assert!((&p - &v).is_equal(&expected))
    }

    #[test]
    #[should_panic(expected = "Cannot subtract a point from a vector")]
    fn subtracting_point_from_vector() {
        let p = Tuple::point(3.0, 2.0, 1.0);
        let v = Tuple::vector(5.0, 6.0, 7.0);
        // let expected = Tuple::point(-2.0, -4.0, -6.0);
        let _ = &v - &p;
    }

    #[test]
    fn negating_a_tuple() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let expected = Tuple::new(-1.0, 2.0, -3.0, 4.0);
        assert!((&a.neg()).is_equal(&expected));
        assert!((-&a).is_equal(&expected));
    }

    #[test]
    fn multiplying_tuple_by_scalar() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let expected = Tuple::new(3.5, -7.0, 10.5, -14.0);
        assert!((&a * 3.5).is_equal(&expected));
    }

    #[test]
    fn multiplying_tuple_by_fraction() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let expected = Tuple::new(0.5, -1.0, 1.5, -2.0);
        assert!((&a * 0.5).is_equal(&expected));
    }

    #[test]
    fn dividing_tuple_by_scalar() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let expected = Tuple::new(0.5, -1.0, 1.5, -2.0);
        assert!((&a / 2.0).is_equal(&expected));
    }

    #[test]
    #[should_panic(expected = "Attempted to divide tuple by zero")]
    fn dividing_tuple_by_zero() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let _ = &a / 0.0;
    }

    #[test]
    fn magnitude_of_vector() {
        let v = Tuple::vector(1.0, 0.0, 0.0);
        assert!(crate::utils::equal(v.magnitude(), 1.0));
    }
    
    #[test]
    fn magnitude_of_vector_123() {
        let v = Tuple::vector(1.0, 2.0, 3.0);
        assert!(crate::utils::equal(v.magnitude(), 14.0_f64.sqrt()));
    }
    
    #[test]
    fn normalize_vector() {
        let v = Tuple::vector(4.0, 0.0, 0.0);
        let n = v.normalize();
        let expected = Tuple::vector(1.0, 0.0, 0.0);
        assert!(n.is_equal(&expected));
    }
    
    #[test]
    fn normalized_vector_has_magnitude_one() {
        let v = Tuple::vector(1.0, 2.0, 3.0);
        let n = v.normalize();
        assert!(crate::utils::equal(n.magnitude(), 1.0));
    }

    #[test]
    fn dot_product_of_two_vectors() {
        let a = Tuple::vector(1.0, 2.0, 3.0);
        let b = Tuple::vector(2.0, 3.0, 4.0);
        assert!(crate::utils::equal(a.dot(&b), 20.0));
    }

    #[test]
    fn cross_product_of_two_vectors() {
        let a = Tuple::vector(1.0, 2.0, 3.0);
        let b = Tuple::vector(2.0, 3.0, 4.0);
        let expected = Tuple::vector(-1.0, 2.0, -1.0);
        assert!((&a.cross(&b)).is_equal(&expected));
        assert!((&b.cross(&a)).is_equal(&(-&expected)));
    }
}