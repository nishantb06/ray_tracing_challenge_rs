use crate::tuple::Tuple;
use crate::ray::Ray;
use crate::intersection::{Intersection, Intersections};
use crate::shape::{ShapeData,Shape};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Sphere {
    pub data: ShapeData,
    // no center/radius needed — unit sphere at origin in object space
}

impl Sphere {
    pub fn new() -> Self {
        Sphere {
            data: ShapeData::new(),
        }
    }

    pub fn glass_sphere() -> Self {
        let mut s = Sphere { data: ShapeData::new() };
        let m = s.material_mut();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        return s;
    }
}

impl Shape for Sphere {
    fn shape_data(&self) -> &ShapeData { &self.data }
    fn shape_data_mut(&mut self) -> &mut ShapeData { &mut self.data }

    fn local_intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        // ray is already in object space — just the math, no transforms here
        let sphere_to_ray = &ray.origin - &Tuple::point(0.0, 0.0, 0.0);
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 { return Intersections::new(vec![]); }
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        Intersections::new(vec![
            Intersection::new(t1, self),
            Intersection::new(t2, self),
        ])
    }

    fn local_normal_at(&self, local_point: &Tuple) -> Tuple {
        // Just the vector from origin — no transforms, normal_at handles those
        local_point - &Tuple::point(0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Material;
    use crate::ray::Ray;
    use crate::tuple::Tuple;
    use crate::matrix::Matrix;

    #[test]
    fn sphere_is_a_shape() {
        fn assert_is_shape<T: Shape>(_: &T) {}
        let s = Sphere::new();
        assert_is_shape(&s);
    }

    #[test]
    fn a_sphere_has_unique_id() {
        let s1 = Sphere::new();
        let s2 = Sphere::new();
        assert_ne!(s1.data.id, s2.data.id);
    }

    // --- local_intersect tests (ray already in object space) ---

    #[test]
    fn a_ray_intersects_a_sphere_at_two_points() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 4.0));
        assert!(crate::utils::equal(xs.data[1].t, 6.0));
    }

    #[test]
    fn a_ray_intersects_a_sphere_at_a_tangent() {
        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 5.0));
        assert!(crate::utils::equal(xs.data[1].t, 5.0));
    }

    #[test]
    fn a_ray_misses_a_sphere() {
        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 0);
    }

    #[test]
    fn a_ray_originates_inside_a_sphere() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, -1.0));
        assert!(crate::utils::equal(xs.data[1].t, 1.0));
    }

    #[test]
    fn a_sphere_is_behind_a_ray() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, -6.0));
        assert!(crate::utils::equal(xs.data[1].t, -4.0));
    }

    #[test]
    fn intersections_are_returned_in_increasing_order() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert!(xs.data[0].t <= xs.data[1].t,
            "expected xs[0].t <= xs[1].t, got {} > {}", xs.data[0].t, xs.data[1].t);
    }

    #[test]
    fn intersect_sets_the_object_on_the_intersection() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.local_intersect(&r);
        assert_eq!(xs.count(), 2);
        assert_eq!(xs.data[0].object.id(), s.data.id);
        assert_eq!(xs.data[1].object.id(), s.data.id);
    }

    // --- local_normal_at tests (point already in object space) ---

    #[test]
    fn normal_on_sphere_at_point_on_x_axis() {
        let s = Sphere::new();
        let n = s.local_normal_at(&Tuple::point(1.0, 0.0, 0.0));
        assert!(n.is_equal(&Tuple::vector(1.0, 0.0, 0.0)));
    }

    #[test]
    fn normal_on_sphere_at_point_on_y_axis() {
        let s = Sphere::new();
        let n = s.local_normal_at(&Tuple::point(0.0, 1.0, 0.0));
        assert!(n.is_equal(&Tuple::vector(0.0, 1.0, 0.0)));
    }

    #[test]
    fn normal_on_sphere_at_point_on_z_axis() {
        let s = Sphere::new();
        let n = s.local_normal_at(&Tuple::point(0.0, 0.0, 1.0));
        assert!(n.is_equal(&Tuple::vector(0.0, 0.0, 1.0)));
    }

    #[test]
    fn normal_on_sphere_at_nonaxial_point() {
        let s = Sphere::new();
        let v = (3.0_f64).sqrt() / 3.0;
        let n = s.local_normal_at(&Tuple::point(v, v, v));
        assert!(n.is_equal(&Tuple::vector(v, v, v)));
    }

    #[test]
    fn normal_is_a_normalized_vector() {
        let s = Sphere::new();
        let v = (3.0_f64).sqrt() / 3.0;
        let n = s.local_normal_at(&Tuple::point(v, v, v));
        assert!(n.is_equal(&n.normalize()));
    }

    // These two keep normal_at() since they test the world-space transform
    // pipeline — exactly what normal_at() is responsible for. Replacing
    // them with local_normal_at() would test nothing since there is no
    // transform involved at the local level.
    #[test]
    fn normal_on_a_translated_sphere() {
        let mut s = Sphere::new();
        s.set_transform(crate::transformation::translation(0.0, 1.0, 0.0));
        let n = s.normal_at(&Tuple::point(0.0, 1.70711, -0.70711));
        assert!(n.is_equal(&Tuple::vector(0.0, 0.70711, -0.70711)));
    }

    #[test]
    fn normal_on_a_transformed_sphere() {
        let mut s = Sphere::new();
        let m = &crate::transformation::scaling(1.0, 0.5, 1.0)
              * &crate::transformation::rotation_z(std::f64::consts::PI / 5.0);
        s.set_transform(m);
        let n = s.normal_at(&Tuple::point(
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        ));
        assert!(n.is_equal(&Tuple::vector(0.0, 0.97014, -0.24254)));
    }

    #[test]
    fn a_sphere_has_a_default_material() {
        let s = Sphere::new();
        assert_eq!(s.material(), &Material::new());
    }

    #[test]
    fn a_sphere_may_be_assigned_a_material() {
        let mut s = Sphere::new();
        s.material_mut().ambient = 1.0;
        assert!(crate::utils::equal(s.material().ambient, 1.0));
    }

    #[test]
    fn glass_sphere_has_glassy_defaults() {
        let s = Sphere::glass_sphere();

        // Then s.transform = identity_matrix
        assert_eq!(s.transform(), &Matrix::identity(4));

        // And s.material.transparency = 1.0
        assert!(crate::utils::equal(s.material().transparency, 1.0));

        // And s.material.refractive_index = 1.5
        assert!(crate::utils::equal(s.material().refractive_index, 1.5));
    }
}