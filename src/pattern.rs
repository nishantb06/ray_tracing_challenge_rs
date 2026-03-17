use std::fmt::Debug;
use crate::canvas::Color;
use crate::matrix::Matrix;
use crate::shape::Shape;
use crate::tuple::Tuple;

#[derive(Debug, Clone, PartialEq)]
pub struct PatternData {
    pub transform: Matrix,
    pub transform_inverse: Matrix,
}

impl PatternData {
    pub fn new() -> Self {
        PatternData {
            transform: Matrix::identity(4),
            transform_inverse: Matrix::identity(4),
        }
    }

    pub fn set_transform(&mut self, t: Matrix) {
        self.transform_inverse = t.inverse_gauss_jordan();
        self.transform = t;
    }
}

pub trait Pattern: Debug {
    fn pattern_data(&self) -> &PatternData;
    fn pattern_data_mut(&mut self) -> &mut PatternData;
    fn pattern_at(&self, point: &Tuple) -> Color;
    fn pattern_at_shape(&self, shape: &dyn Shape, world_point: &Tuple) -> Color {
        let object_point = &shape.shape_data().transform_inverse * world_point;
        let pattern_point = &self.pattern_data().transform_inverse * &object_point;
        self.pattern_at(&pattern_point)
    }
    fn transform(&self) -> &Matrix { &self.pattern_data().transform }
    fn set_transform(&mut self, t: Matrix) { self.pattern_data_mut().set_transform(t); }
}

#[derive(Debug,Clone,PartialEq)]
pub struct StripePattern {
    pub a: Color,
    pub b: Color,
    pub data: PatternData,
}

impl StripePattern {
    pub fn new(a: Color, b: Color) -> Self {
        StripePattern { a, b, data: PatternData::new() }
    }
}

impl Pattern for StripePattern {
    fn pattern_data(&self) -> &PatternData { &self.data }
    fn pattern_data_mut(&mut self) -> &mut PatternData { &mut self.data }

    fn pattern_at(&self, point: &Tuple) -> Color {
        if point.x.floor() as i64 % 2 == 0 { self.a.clone() } else { self.b.clone() }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct GradientPattern {
    pub a: Color,
    pub b: Color,
    pub data: PatternData,
}

impl GradientPattern {
    pub fn new(a: Color, b: Color) -> Self {
        GradientPattern { a, b, data: PatternData::new() }
    }
}

impl Pattern for GradientPattern {
    fn pattern_data(&self) -> &PatternData { &self.data }
    fn pattern_data_mut(&mut self) -> &mut PatternData { &mut self.data }

    fn pattern_at(&self, point: &Tuple) -> Color {
        let distance = &self.b - &self.a;
        let fraction = point.x - point.x.floor();
        &self.a + &(&distance * fraction)
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct RingPattern {
    pub a: Color,
    pub b: Color,
    pub data: PatternData,
}

impl RingPattern {
    pub fn new(a: Color, b: Color) -> Self {
        RingPattern { a, b, data: PatternData::new() }
    }
}

impl Pattern for RingPattern {
    fn pattern_data(&self) -> &PatternData { &self.data }
    fn pattern_data_mut(&mut self) -> &mut PatternData { &mut self.data }

    fn pattern_at(&self, point: &Tuple) -> Color {
        let radius = (point.x * point.x + point.z * point.z).sqrt();
        if radius.floor() as i64 % 2 == 0 { self.a.clone() } else { self.b.clone() }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct CheckersPattern {
    pub a: Color,
    pub b: Color,
    pub data: PatternData,
}

impl CheckersPattern {
    pub fn new(a: Color, b: Color) -> Self {
        CheckersPattern { a, b, data: PatternData::new() }
    }
}

impl Pattern for CheckersPattern {
    fn pattern_data(&self) -> &PatternData { &self.data }
    fn pattern_data_mut(&mut self) -> &mut PatternData { &mut self.data }

    fn pattern_at(&self, point: &Tuple) -> Color {
        let sum = point.x.floor() + point.y.floor() + point.z.floor();
        if (sum as i64) % 2 == 0 { self.a.clone() } else { self.b.clone() }
    }
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    #[derive(Debug,Clone)]
    pub struct TestPattern {
        pub data: PatternData,
    }
    impl TestPattern {
        pub fn new() -> Self {
            TestPattern { data: PatternData::new() }
        }
    }
    impl Pattern for TestPattern {
        fn pattern_data(&self) -> &PatternData { &self.data}
        fn pattern_data_mut(&mut self) -> &mut PatternData {&mut self.data}

        fn pattern_at(&self, point: &Tuple) -> Color {
            Color::new(point.x, point.y, point.z)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{BLACK, WHITE};
    use crate::sphere::Sphere;
    use crate::transformation::{scaling, translation};
    use crate::tuple::Tuple;
    use super::test_support::TestPattern;

    #[test]
    fn creating_stripe_pattern() {
        let pattern_data = PatternData::new();
        let pattern = StripePattern { a: WHITE, b: BLACK , data: pattern_data};
        assert_eq!(pattern.a, WHITE);
        assert_eq!(pattern.b, BLACK);
    }

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let pattern = StripePattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 1.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 2.0, 0.0)), WHITE);
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let pattern = StripePattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 1.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 2.0)), WHITE);
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = StripePattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.9, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(1.0, 0.0, 0.0)), BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(-0.1, 0.0, 0.0)), BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(-1.0, 0.0, 0.0)), BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(-1.1, 0.0, 0.0)), WHITE);
    }

    #[test]
    fn stripes_with_an_object_transformation() {
        let mut object = Sphere::new();
        object.set_transform(scaling(2.0, 2.0, 2.0));
        let pattern = StripePattern::new(WHITE, BLACK);
        let c = pattern.pattern_at_shape(
            &object,
            &Tuple::point(1.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }

    #[test]
    fn stripes_with_a_pattern_transformation() {
        let object = Sphere::new();
        let mut pattern = StripePattern::new(WHITE, BLACK);
        pattern.set_transform(scaling(2.0, 2.0, 2.0));
        let c = pattern.pattern_at_shape(
            &object,
            &Tuple::point(1.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }

    #[test]
    fn stripes_with_both_object_and_pattern_transformation() {
        let mut object = Sphere::new();
        object.set_transform(scaling(2.0, 2.0, 2.0));
        let mut pattern = StripePattern::new(WHITE, BLACK);
        pattern.set_transform(translation(0.5, 0.0, 0.0));
        let c = pattern.pattern_at_shape(
            &object,
            &Tuple::point(2.5, 0.0, 0.0),
        );
        assert_eq!(c, WHITE);
    }

    #[test]
    fn default_pattern_transformation() {
        let pattern = TestPattern::new();
        assert_eq!(pattern.transform(), &Matrix::identity(4));
    }

    #[test]
    fn assigning_a_pattern_transformation() {
        let mut pattern = TestPattern::new();
        pattern.set_transform(translation(1.0, 2.0, 3.0));
        assert_eq!(pattern.transform(), &translation(1.0, 2.0, 3.0));
    }

    #[test]
    fn pattern_with_object_transformation() {
        let mut shape = Sphere::new();
        shape.set_transform(scaling(2.0, 2.0, 2.0));
        let pattern = TestPattern::new();
        let c = pattern.pattern_at_shape(&shape, &Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(c, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn pattern_with_pattern_transformation() {
        let shape = Sphere::new();
        let mut pattern = TestPattern::new();
        pattern.set_transform(scaling(2.0, 2.0, 2.0));
        let c = pattern.pattern_at_shape(&shape, &Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(c, Color::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn pattern_with_both_object_and_pattern_transformation() {
        let mut shape = Sphere::new();
        shape.set_transform(scaling(2.0, 2.0, 2.0));
        let mut pattern = TestPattern::new();
        pattern.set_transform(translation(0.5, 1.0, 1.5));
        let c = pattern.pattern_at_shape(&shape, &Tuple::point(2.5, 3.0, 3.5));
        assert_eq!(c, Color::new(0.75, 0.5, 0.25));
    }

    // Gradient pattern tests
    #[test]
    fn gradient_linearly_interpolates_between_colors() {
        let pattern = GradientPattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(
            pattern.pattern_at(&Tuple::point(0.25, 0.0, 0.0)),
            Color::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point(0.5, 0.0, 0.0)),
            Color::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            pattern.pattern_at(&Tuple::point(0.75, 0.0, 0.0)),
            Color::new(0.25, 0.25, 0.25)
        );
    }

    // Ring pattern tests
    #[test]
    fn ring_extends_in_both_x_and_z() {
        let pattern = RingPattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(1.0, 0.0, 0.0)), BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 1.0)), BLACK);
        assert_eq!(
            pattern.pattern_at(&Tuple::point(0.708, 0.0, 0.708)),
            BLACK
        );
    }

    // 3D checker pattern tests
    #[test]
    fn checkers_repeat_in_x() {
        let pattern = CheckersPattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.99, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(1.01, 0.0, 0.0)), BLACK);
    }

    #[test]
    fn checkers_repeat_in_y() {
        let pattern = CheckersPattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.99, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 1.01, 0.0)), BLACK);
    }

    #[test]
    fn checkers_repeat_in_z() {
        let pattern = CheckersPattern::new(WHITE, BLACK);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 0.99)), WHITE);
        assert_eq!(pattern.pattern_at(&Tuple::point(0.0, 0.0, 1.01)), BLACK);
    }
}