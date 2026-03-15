use std::sync::atomic::{AtomicU64, Ordering};
use crate::intersection::Intersections;
use crate::matrix::Matrix;
use crate::material::Material;
use crate::ray::Ray;
use crate::tuple::Tuple;

static NEXT_SHAPE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct ShapeData {
    pub id: u64,
    pub transform: Matrix,
    pub material: Material,
    pub transform_inverse: Matrix,  // cache this!
}

impl ShapeData {
    pub fn new() -> Self {
        let transform = Matrix::identity(4);
        let transform_inverse = Matrix::identity(4);
        ShapeData {
            id: NEXT_SHAPE_ID.fetch_add(1, Ordering::Relaxed),
            transform,
            material: Material::new(),
            transform_inverse,
        }
    }
    pub fn set_transform(&mut self, t: Matrix) {
        self.transform_inverse = t.inverse_gauss_jordan();
        self.transform = t;
    }
}

pub trait Shape {
    fn shape_data(&self) -> &ShapeData;
    fn shape_data_mut(&mut self) -> &mut ShapeData;

    // Shapes implement these two in object space only:
    fn local_intersect<'a>(&'a self, local_ray: &Ray) -> Intersections<'a>;
    fn local_normal_at(&self, local_point: &Tuple) -> Tuple;

    // These are free default impls — Sphere/Plane get them for free:
    fn intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
        let local_ray = ray.transform(&self.shape_data().transform_inverse);
        self.local_intersect(&local_ray)
    }

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let sd = self.shape_data();
        let local_point = &sd.transform_inverse * world_point;
        let local_normal = self.local_normal_at(&local_point);
        let mut world_normal = &sd.transform_inverse.transpose() * &local_normal;
        world_normal.w = 0.0;
        world_normal.normalize()
    }

    fn id(&self) -> u64 { self.shape_data().id }
    fn transform(&self) -> &Matrix { &self.shape_data().transform }
    fn material(&self) -> &Material { &self.shape_data().material }
    fn material_mut(&mut self) -> &mut Material { &mut self.shape_data_mut().material }
    fn set_transform(&mut self, t: Matrix) { self.shape_data_mut().set_transform(t); }
}

#[cfg(test)]
#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::RefCell;

    pub struct TestShape {
        pub data: ShapeData,
        pub saved_ray: RefCell<Option<Ray>>,
    }

    impl TestShape {
        pub fn new() -> Self {
            TestShape {
                data: ShapeData::new(),
                saved_ray: RefCell::new(None),
            }
        }
    }

    impl Shape for TestShape {
        fn shape_data(&self) -> &ShapeData { &self.data }
        fn shape_data_mut(&mut self) -> &mut ShapeData { &mut self.data }

        fn local_intersect<'a>(&'a self, local_ray: &Ray) -> Intersections<'a> {
            *self.saved_ray.borrow_mut() = Some(local_ray.clone());
            Intersections::new(vec![])
        }

        fn local_normal_at(&self, local_point: &Tuple) -> Tuple {
            Tuple::vector(local_point.x, local_point.y, local_point.z)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_support::TestShape;
    use crate::transformation::{translation, scaling, rotation_z};
    use crate::material::Material;
    use crate::ray::Ray;
    use crate::tuple::Tuple;

    fn test_shape() -> TestShape {
        TestShape::new()
    }

    #[test]
    fn default_transformation() {
        let s = test_shape();
        assert_eq!(s.transform(), &Matrix::identity(4));
    }

    #[test]
    fn assigning_a_transformation() {
        let mut s = test_shape();
        s.set_transform(translation(2.0, 3.0, 4.0));
        assert_eq!(s.transform(), &translation(2.0, 3.0, 4.0));
    }

    #[test]
    fn default_material() {
        let s = test_shape();
        assert_eq!(s.material(), &Material::new());
    }

    #[test]
    fn assigning_a_material() {
        let mut s = test_shape();
        let mut m = Material::new();
        m.ambient = 1.0;
        s.material_mut().ambient = 1.0;
        assert_eq!(s.material(), &m);
    }

    #[test]
    fn intersecting_scaled_shape_with_ray() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = test_shape();
        s.set_transform(scaling(2.0, 2.0, 2.0));
        let _xs = s.intersect(&r);
        let saved = s.saved_ray.borrow();
        let saved_ray = saved.as_ref().expect("saved_ray should be set after intersect");
        assert!(saved_ray.origin.is_equal(&Tuple::point(0.0, 0.0, -2.5)));
        assert!(saved_ray.direction.is_equal(&Tuple::vector(0.0, 0.0, 0.5)));
    }

    #[test]
    fn intersecting_translated_shape_with_ray() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = test_shape();
        s.set_transform(translation(5.0, 0.0, 0.0));
        let _xs = s.intersect(&r);
        let saved = s.saved_ray.borrow();
        let saved_ray = saved.as_ref().expect("saved_ray should be set after intersect");
        assert!(saved_ray.origin.is_equal(&Tuple::point(-5.0, 0.0, -5.0)));
        assert!(saved_ray.direction.is_equal(&Tuple::vector(0.0, 0.0, 1.0)));
    }

    #[test]
    fn normal_on_translated_shape() {
        let mut s = test_shape();
        s.set_transform(translation(0.0, 1.0, 0.0));
        let n = s.normal_at(&Tuple::point(0.0, 1.70711, -0.70711));
        assert!(n.is_equal(&Tuple::vector(0.0, 0.70711, -0.70711)));
    }

    #[test]
    fn normal_on_transformed_shape() {
        let mut s = test_shape();
        let m = &scaling(1.0, 0.5, 1.0)
              * &rotation_z(std::f64::consts::PI / 5.0);
        s.set_transform(m);
        let n = s.normal_at(&Tuple::point(
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        ));
        assert!(n.is_equal(&Tuple::vector(0.0, 0.97014, -0.24254)));
    }
}