use crate::canvas::Canvas;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::tuple::Tuple;
use crate::world::{World, color_at};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Camera {
    pub hsize: usize,
    pub vsize: usize,
    pub field_of_view: f64,
    pub transform: Matrix,
    pub pixel_size: f64,
    pub half_width: f64,
    pub half_height: f64,
}

#[allow(dead_code)]
impl Camera {
    pub fn ray_for_pixel(&self, px: usize, py: usize) -> Ray {
        let x_offset = (px as f64 + 0.5) * self.pixel_size;
        let y_offset = (py as f64 + 0.5) * self.pixel_size;
        let world_x = self.half_width - x_offset;
        let world_y = self.half_height - y_offset;
        let inv = self.transform.inverse_gauss_jordan();
        let pixel = &inv * &Tuple::point(world_x, world_y, -1.0);
        let origin = &inv * &Tuple::point(0.0, 0.0, 0.0);
        let mut direction = &pixel - &origin;
        direction.w = 0.0;
        let direction = direction.normalize();
        Ray::new(origin, direction)
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);
        for y in 0..self.vsize {
            for x in 0..self.hsize {
                let ray = self.ray_for_pixel(x, y);
                let color = color_at(world, &ray);
                image.write_pixel(x, y, color);
            }
        }
        image
    }

    pub fn new(hsize: usize, vsize: usize, field_of_view: f64) -> Self {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize as f64 / vsize as f64;
        let (half_width, half_height) = if aspect >= 1.0 {
            (half_view, half_view / aspect)
        } else {
            (half_view * aspect, half_view)
        };
        let pixel_size = (half_width * 2.0) / hsize as f64;

        Camera {
            hsize,
            vsize,
            field_of_view,
            transform: Matrix::identity(4),
            pixel_size,
            half_width,
            half_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Color;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn constructing_a_camera() {
        let hsize = 160;
        let vsize = 120;
        let field_of_view = FRAC_PI_2; // π/2
        let c = Camera::new(hsize, vsize, field_of_view);

        assert_eq!(c.hsize, 160);
        assert_eq!(c.vsize, 120);
        assert!(crate::utils::equal(c.field_of_view, FRAC_PI_2));
        assert!(c.transform == Matrix::identity(4));
    }

    #[test]
    fn pixel_size_for_horizontal_canvas() {
        let c = Camera::new(200, 125, FRAC_PI_2);
        assert!(crate::utils::equal(c.pixel_size, 0.01));
    }

    #[test]
    fn pixel_size_for_vertical_canvas() {
        let c = Camera::new(125, 200, FRAC_PI_2);
        assert!(crate::utils::equal(c.pixel_size, 0.01));
    }

    #[test]
    fn ray_through_center_of_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);
        let r = c.ray_for_pixel(100, 50);
        assert!(
            r.origin
                .is_equal(&crate::tuple::Tuple::point(0.0, 0.0, 0.0))
        );
        assert!(
            r.direction
                .is_equal(&crate::tuple::Tuple::vector(0.0, 0.0, -1.0))
        );
    }

    #[test]
    fn ray_through_corner_of_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);
        let r = c.ray_for_pixel(0, 0);
        assert!(
            r.origin
                .is_equal(&crate::tuple::Tuple::point(0.0, 0.0, 0.0))
        );
        assert!(
            r.direction
                .is_equal(&crate::tuple::Tuple::vector(0.66519, 0.33259, -0.66851))
        );
    }

    #[test]
    fn ray_when_camera_is_transformed() {
        use crate::transformation::{rotation_y, translation};
        use std::f64::consts::FRAC_PI_4;
        let mut c = Camera::new(201, 101, FRAC_PI_2);
        c.transform = &rotation_y(FRAC_PI_4) * &translation(0.0, -2.0, 5.0);
        let r = c.ray_for_pixel(100, 50);
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!(
            r.origin
                .is_equal(&crate::tuple::Tuple::point(0.0, 2.0, -5.0))
        );
        assert!(r.direction.is_equal(&crate::tuple::Tuple::vector(
            sqrt2_over_2,
            0.0,
            -sqrt2_over_2
        )));
    }

    #[test]
    fn rendering_a_world_with_a_camera() {
        use crate::transformation::view_transform;
        use crate::world::World;
        let w = World::default_world();
        let mut c = Camera::new(11, 11, FRAC_PI_2);
        let from = Tuple::point(0.0, 0.0, -5.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = view_transform(&from, &to, &up);
        let image = c.render(&w);
        assert!(
            image
                .pixel_at(5, 5)
                .is_equal(&Color::new(0.38066, 0.47583, 0.2855))
        );
    }
}
