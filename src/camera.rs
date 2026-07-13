use crate::canvas::{Canvas,Color};
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::tuple::Tuple;
use crate::world::{World, color_at};
use crate::utils::MAX_RECURSION_DEPTH;
use std::thread;

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

    pub fn ray_for_pixel(&self, px: f64, py: f64) -> Ray {
        // the offset from the edge of the canvas to the pixel's center
        let xoffset = (px + 0.5) * self.pixel_size;
        let yoffset = (py + 0.5) * self.pixel_size;

        // the untransformed coordinates of the pixel in world space.
        // (remember that the camera looks toward -z, so +x is to the *left*.)
        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        // using the camera matrix, transform the canvas point and the origin,
        // and then compute the ray's direction vector.
        // (remember that the canvas is at z=-1)
        let inv = self.transform.inverse_gauss_jordan();
        let pixel = &inv * &Tuple::point(world_x, world_y, -1.0);
        let origin = &inv * &Tuple::point(0.0, 0.0, 0.0);
        let direction = (&pixel - &origin).normalize();

        return Ray { origin, direction };
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);
        
        let coordinates: Vec<(usize, usize)> = (0..self.hsize)
            .flat_map(|x| (0..self.vsize).map(move |y| (x, y)))
            .collect();
            
        // let handles: Vec<_> = thread::scope(|scope| {
        //     coordinates
        //         .iter()
        //         .map(|&(x, y)| {
        //             scope.spawn(move || {
        //                 let ray = self.ray_for_pixel(x as f64, y as f64);
        //                 let color = color_at(world, &ray, MAX_RECURSION_DEPTH);
        //                 color
        //             })
        //         })
        //         .collect::<Vec<_>>()
        // }); // scope blocks here until all threads finish
        
        // let colors: Vec<Color> = handles
        //     .into_iter()
        //     .map(|handle| handle.join().unwrap())
        //     .collect();
        let colors: Vec<Color> = thread::scope(|scope| {
            let handles: Vec<_> = coordinates
                .iter()
                .map(|&(x, y)| {
                    scope.spawn(move || {
                        let ray = self.ray_for_pixel(x as f64, y as f64);
                        color_at(world, &ray, MAX_RECURSION_DEPTH)
                    })
                })
                .collect();
        
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect()
        });
        // for y in 0..self.vsize {
        //     for x in 0..self.hsize {
        //         let ray = self.ray_for_pixel(x as f64, y as f64);
        //         let color = color_at(world, &ray, MAX_RECURSION_DEPTH);
        //         image.write_pixel(x, y, color);
        //     }
        // }
        for (&(x, y), color) in coordinates.iter().zip(colors) {
            image.write_pixel(x, y, color);
        }
        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Color;
    use crate::transformation::{rotation_y, translation};
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
    fn constructing_a_ray_through_the_center_of_the_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);

        let r = c.ray_for_pixel(100.0, 50.0);

        assert!(r.origin == Tuple::point(0.0, 0.0, 0.0));

        let expected = Tuple::vector(0.0, 0.0, -1.0);

        assert!(crate::utils::equal(r.direction.x, expected.x));
        assert!(crate::utils::equal(r.direction.y, expected.y));
        assert!(crate::utils::equal(r.direction.z, expected.z));
    }

    #[test]
    fn constructing_a_ray_through_a_corner_of_the_canvas() {
        let c = Camera::new(201, 101, FRAC_PI_2);

        let r = c.ray_for_pixel(0.0, 0.0);

        assert!(r.origin == Tuple::point(0.0, 0.0, 0.0));

        let expected = Tuple::vector(0.66519, 0.33259, -0.66851);

        assert!(crate::utils::equal(r.direction.x, expected.x));
        assert!(crate::utils::equal(r.direction.y, expected.y));
        assert!(crate::utils::equal(r.direction.z, expected.z));
    }

    #[test]
    fn constructing_a_ray_when_the_camera_is_transformed() {
        use std::f64::consts::{FRAC_PI_4, SQRT_2};

        let mut c = Camera::new(201, 101, FRAC_PI_2);

        c.transform = &rotation_y(FRAC_PI_4) * &translation(0.0, -2.0, 5.0);

        let r = c.ray_for_pixel(100.0, 50.0);

        assert!(r.origin == Tuple::point(0.0, 2.0, -5.0));

        let expected = Tuple::vector(SQRT_2 / 2.0, 0.0, -SQRT_2 / 2.0);

        assert!(crate::utils::equal(r.direction.x, expected.x));
        assert!(crate::utils::equal(r.direction.y, expected.y));
        assert!(crate::utils::equal(r.direction.z, expected.z));
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
