
use std::fmt::Debug;
use crate::canvas::Color;

#[derive(Debug,Clone,PartialEq)]
pub struct StripePattern {
    pub a: Color,
    pub b: Color,
}

impl StripePattern {
    pub fn new(a: Color, b: Color) -> Self {
        Self { a, b }
    }

    pub fn stripe_at(&self, x: f64, _y: f64, _z: f64) -> Color {
        if x.floor() as i64 % 2 == 0 {
            self.a.clone()
        } else {
            self.b.clone()
        }
    }
}

#[cfg(test)]
pub mod test_support {
    use crate::canvas::Color;

    pub const BLACK: Color = Color { red: 0.0, green: 0.0, blue: 0.0 };
    pub const WHITE: Color = Color { red: 1.0, green: 1.0, blue: 1.0 };
    
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::test_support::{BLACK,WHITE};

    #[test]
    fn creating_stripe_pattern() {
        let pattern = StripePattern { a: WHITE, b: BLACK };
        assert_eq!(pattern.a, WHITE);
        assert_eq!(pattern.b, BLACK);
    }

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let pattern = StripePattern { a: WHITE, b: BLACK };
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 1.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 2.0, 0.0), WHITE);
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let pattern = StripePattern { a: WHITE, b: BLACK };
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 0.0, 1.0), WHITE);
        assert_eq!(pattern.stripe_at(0.0, 0.0, 2.0), WHITE);
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let pattern = StripePattern { a: WHITE, b: BLACK };
        assert_eq!(pattern.stripe_at(0.0, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(0.9, 0.0, 0.0), WHITE);
        assert_eq!(pattern.stripe_at(1.0, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-0.1, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-1.0, 0.0, 0.0), BLACK);
        assert_eq!(pattern.stripe_at(-1.1, 0.0, 0.0), WHITE);
    }
}