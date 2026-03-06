use std::ops::{Add, Sub, Mul};
use crate::utils::equal;

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

#[allow(dead_code)]
impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        equal(self.red, other.red)
            && equal(self.green, other.green)
            && equal(self.blue, other.blue)
    }

    /// Hadamard product: component-wise multiplication of two colors
    pub fn hadamard_product(&self, other: &Self) -> Self {
        Self {
            red: self.red * other.red,
            green: self.green * other.green,
            blue: self.blue * other.blue,
        }
    }
}

impl Add for &Color {
    type Output = Color;
    fn add(self, rhs: Self) -> Self::Output {
        Color { red: self.red + rhs.red, green: self.green + rhs.green, blue: self.blue + rhs.blue }
    }
}

impl Sub for &Color {
    type Output = Color;
    fn sub(self, rhs: Self) -> Self::Output {
        Color { red: self.red - rhs.red, green: self.green - rhs.green, blue: self.blue - rhs.blue }
    }
}

impl Mul<f64> for &Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        Color { red: self.red * rhs, green: self.green * rhs, blue: self.blue * rhs }
    }
}

impl Mul<&Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: &Color) -> Self::Output { rhs * self }
}

impl Mul for &Color {
    type Output = Color;
    fn mul(self, rhs: Self) -> Self::Output {
        self.hadamard_product(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_creates_tuple_with_red_green_blue() {
        let c = Color::new(-0.5, 0.4, 1.7);
        assert!(equal(c.red, -0.5));
        assert!(equal(c.green, 0.4));
        assert!(equal(c.blue, 1.7));
    }

    #[test]
    fn adding_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        let expected = Color::new(1.6, 0.7, 1.0);
        assert!((&c1 + &c2).is_equal(&expected));
    }

    #[test]
    fn subtracting_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        let expected = Color::new(0.2, 0.5, 0.5);
        assert!((&c1 - &c2).is_equal(&expected));
    }

    #[test]
    fn multiplying_color_by_scalar() {
        let c = Color::new(0.2, 0.3, 0.4);
        let expected = Color::new(0.4, 0.6, 0.8);
        assert!((&c * 2.0).is_equal(&expected));
    }

    #[test]
    fn multiplying_colors() {
        let c1 = Color::new(1.0, 0.2, 0.4);
        let c2 = Color::new(0.9, 1.0, 0.1);
        let expected = Color::new(0.9, 0.2, 0.04);
        assert!((&c1 * &c2).is_equal(&expected));
    }
}