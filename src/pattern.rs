use std::fmt::Debug;
use crate::canvas::Color;
use crate::matrix::Matrix;
use crate::shape::Shape;

#[derive(Debug,Clone,PartialEq)]
pub struct StripePattern {
    pub a: Color,
    pub b: Color,
    transform: Matrix,
    transform_inverse: Matrix,
}

impl StripePattern {
    pub fn new(a: Color, b: Color) -> Self {
        Self { a, b, transform: Matrix::identity(4), transform_inverse: Matrix::identity(4) }
    }

    pub fn stripe_at(&self, x: f64, _y: f64, _z: f64) -> Color {
        if x.floor() as i64 % 2 == 0 {
            self.a.clone()
        } else {
            self.b.clone()
        }
    }

    pub fn set_transform(&mut self, t: Matrix) {
        self.transform_inverse = t.inverse_gauss_jordan();
        self.transform = t;
    }

    /// Return the color for the given pattern on the given object at the given world-space point.
    /// Respects both the object's and the pattern's transforms.
    pub fn stripe_at_object<S: Shape + ?Sized>(pattern: &StripePattern, object: &S, point: crate::tuple::Tuple) -> Color {
        let object_point = &object.shape_data().transform_inverse * &point;
        let pattern_point = &pattern.transform_inverse * &object_point;
        pattern.stripe_at(pattern_point.x, pattern_point.y, pattern_point.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{BLACK, WHITE};
    use crate::sphere::Sphere;
    use crate::transformation::{scaling, translation};
    use crate::tuple::Tuple;

    #[test]
    fn creating_stripe_pattern() {
        let pattern = StripePattern { a: WHITE, b: BLACK ,transform: Matrix::identity(4), transform_inverse: Matrix::identity(4)};
        assert_eq!(pattern.a, WHITE);
        assert_eq!(pattern.b, BLACK);
    }

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let pattern = StripePattern { a: WHITE, b: BLACK ,transform: Matrix::identity(4), transform_inverse: Matrix::identity(4)};
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 1.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 2.0, 0.0), WHITE);
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let pattern = StripePattern { a: WHITE, b: BLACK,transform: Matrix::identity(4), transform_inverse: Matrix::identity(4) };
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 0.0, 1.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 0.0, 2.0), WHITE);
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = StripePattern { a: WHITE, b: BLACK ,transform: Matrix::identity(4), transform_inverse: Matrix::identity(4)};
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.9, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(1.0, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-0.1, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-1.0, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-1.1, 0.0, 0.0), WHITE);
    }

    #[test]
    fn stripes_with_an_object_transformation() {
        let mut object = Sphere::new();
        object.set_transform(scaling(2.0, 2.0, 2.0));
        let pattern = StripePattern::new(WHITE, BLACK);
        let c = StripePattern::stripe_at_object(
            &pattern,
            &object,
            Tuple::point(1.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }

    #[test]
    fn stripes_with_a_pattern_transformation() {
        let object = Sphere::new();
        let mut pattern = StripePattern::new(WHITE, BLACK);
        pattern.set_transform(scaling(2.0, 2.0, 2.0));
        let c = StripePattern::stripe_at_object(
            &pattern,
            &object,
            Tuple::point(1.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }

    #[test]
    fn stripes_with_both_object_and_pattern_transformation() {
        let mut object = Sphere::new();
        object.set_transform(scaling(2.0, 2.0, 2.0));
        let mut pattern = StripePattern::new(WHITE, BLACK);
        pattern.set_transform(translation(0.5, 0.0, 0.0));
        let c = StripePattern::stripe_at_object(
            &pattern,
            &object,
            Tuple::point(2.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }
}