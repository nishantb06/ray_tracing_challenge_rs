use std::ops::{Add, Sub, Mul};
use crate::utils::equal;

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
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

pub const WHITE: Color = Color { red: 1.0, green: 1.0, blue: 1.0 };
pub const BLACK: Color = Color { red: 0.0, green: 0.0, blue: 0.0 };


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

#[allow(dead_code)]
impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![Color::new(0.0, 0.0, 0.0); width * height] }
    }

    /// Writes a color to the pixel at (x, y).
    /// x is the column (0..width), y is the row (0..height).
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let idx = y * self.width + x;
        self.pixels[idx] = color;
    }

    /// Returns the color at pixel (x, y).
    pub fn pixel_at(&self, x: usize, y: usize) -> &Color {
        let idx = y * self.width + x;
        &self.pixels[idx]
    }

    /// Mutable row-major pixel slice (index = y * width + x).
    pub fn pixels_mut(&mut self) -> &mut [Color] {
        &mut self.pixels
    }

    /// Converts a color component from [0, 1] to [0, 255], clamped.
    fn scale_component(c: f64) -> u8 {
        let scaled = (c * 255.0).round();
        scaled.clamp(0.0, 255.0) as u8
    }

    pub fn canvas_to_ppm(&self) -> String {
        const MAX_COLOR: u8 = 255;
        const MAX_LINE_LEN: usize = 70;
    
        let mut out = format!("P3\n{} {}\n{}\n", self.width, self.height, MAX_COLOR);
    
        let mut line_len = 0;
    
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = &self.pixels[y * self.width + x];
                let r = Self::scale_component(pixel.red).to_string();
                let g = Self::scale_component(pixel.green).to_string();
                let b = Self::scale_component(pixel.blue).to_string();
    
                for val in [&r, &g, &b] {
                    let addition = if line_len == 0 { val.len() } else { 1 + val.len() };
    
                    if line_len > 0 && line_len + addition > MAX_LINE_LEN {
                        out.push('\n');
                        line_len = 0;
                    }
                    if line_len > 0 {
                        out.push(' ');
                        line_len += 1;
                    }
                    out.push_str(val);
                    line_len += val.len();
                }
            }
            // Each row is terminated by a newline
            out.push('\n');
            line_len = 0;
        }
    
        out
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

    #[test]
    fn creating_a_canvas() {
        let c = Canvas::new(10, 20);
        assert_eq!(c.width, 10);
        assert_eq!(c.height, 20);
        let black = Color::new(0.0, 0.0, 0.0);
        for pixel in &c.pixels {
            assert!(pixel.is_equal(&black), "every pixel should be color(0, 0, 0)");
        }
    }

    #[test]
    fn writing_pixels_to_canvas() {
        let mut c = Canvas::new(10, 20);
        let red = Color::new(1.0, 0.0, 0.0);
        c.write_pixel(2, 3, red.clone());
        assert!(c.pixel_at(2, 3).is_equal(&red));
    }

    #[test]
    fn constructing_the_ppm_header() {
        let c = Canvas::new(5, 3);
        let ppm = c.canvas_to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(lines[0], "P3");
        assert_eq!(lines[1], "5 3");
        assert_eq!(lines[2], "255");
    }

    #[test]
    fn constructing_the_ppm_pixel_data() {
        let mut c = Canvas::new(5, 3);
        let c1 = Color::new(1.5, 0.0, 0.0);
        let c2 = Color::new(0.0, 0.5, 0.0);
        let c3 = Color::new(-0.5, 0.0, 1.0);
        c.write_pixel(0, 0, c1);
        c.write_pixel(2, 1, c2);
        c.write_pixel(4, 2, c3);
        let ppm = c.canvas_to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(lines[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(lines[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(lines[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }

    #[test]
    fn splitting_long_lines_in_ppm_files() {
        let mut c = Canvas::new(10, 2);
        let color = Color::new(1.0, 0.8, 0.6);
        for y in 0..c.height {
            for x in 0..c.width {
                c.write_pixel(x, y, color.clone());
            }
        }
        let ppm = c.canvas_to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(lines[3], "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204");
        assert_eq!(lines[4], "153 255 204 153 255 204 153 255 204 153 255 204 153");
        assert_eq!(lines[5], "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204");
        assert_eq!(lines[6], "153 255 204 153 255 204 153 255 204 153 255 204 153");
    }

    #[test]
    fn ppm_files_are_terminated_by_newline() {
        let c = Canvas::new(5, 3);
        let ppm = c.canvas_to_ppm();
        assert!(ppm.ends_with('\n'), "PPM should end with a newline character");
    }
}