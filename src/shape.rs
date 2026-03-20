use std::sync::atomic::{AtomicU64, Ordering};
use crate::intersection::Intersections;
use crate::matrix::Matrix;
use crate::material::Material;
use crate::ray::Ray;
use crate::tuple::Tuple;
use std::fmt::Debug;

static NEXT_SHAPE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct ShapeData {
    pub id: u64,
    pub transform: Matrix,
    pub material: Material,
    pub transform_inverse: Matrix,  // cache this!
    pub parent: Option<u64>,
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
            parent: None,
        }
    }
    pub fn set_transform(&mut self, t: Matrix) {
        self.transform_inverse = t.inverse_gauss_jordan();
        self.transform = t;
    }
}


pub trait Shape: Debug {
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

/// Recursively convert a **world-space** point into `shape`'s object space.
/// `resolve_parent` must return the parent shape when `shape_data().parent` is `Some(id)`.
pub fn world_to_object<'a>(
    shape: &'a dyn Shape,
    resolve_parent: &impl Fn(u64) -> Option<&'a dyn Shape>,
    point: &Tuple,
) -> Tuple {
    let p = match shape.shape_data().parent {
        Some(parent_id) => {
            let parent = resolve_parent(parent_id)
                .expect("world_to_object: parent id present but not resolvable");
            world_to_object(parent, resolve_parent, point)
        }
        None => point.clone(),
    };
    &shape.shape_data().transform_inverse * &p
}

/// Recursively transform a **object-space** normal into **world space**,
/// walking up the parent chain. Uses cached `transform_inverse` and
/// `transpose(transform_inverse)` ≡ transpose(inverse(transform)) on the linear part.
pub fn normal_to_world<'a>(
    shape: &'a dyn Shape,
    resolve_parent: &impl Fn(u64) -> Option<&'a dyn Shape>,
    normal: &Tuple,
) -> Tuple {
    let mut n = &shape.shape_data().transform_inverse.transpose() * normal;
    n.w = 0.0;
    n = n.normalize();
    if let Some(parent_id) = shape.shape_data().parent {
        let parent = resolve_parent(parent_id)
            .expect("normal_to_world: parent id present but not resolvable");
        normal_to_world(parent, resolve_parent, &n)
    } else {
        n
    }
}

pub fn shape_normal_at<'a>(
    shape: &'a dyn Shape,
    resolve_parent: &impl Fn(u64) -> Option<&'a dyn Shape>,
    world_point: &Tuple,
) -> Tuple {
    let local_point = world_to_object(shape, resolve_parent, world_point);
    let local_normal = shape.local_normal_at(&local_point);
    normal_to_world(shape, resolve_parent, &local_normal)
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug)]
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
    use crate::group::Group;
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

    #[test]
    fn shape_has_a_parent_attribute() {
        let s = test_shape();
        assert_eq!(s.data.parent, None);
    }

    #[test]
    fn converting_a_point_from_world_to_object_space() {
        use crate::shape::world_to_object; // pub fn in shape.rs
        use crate::sphere::Sphere;
        use crate::transformation::{rotation_y, scaling, translation};
    
        let mut g1 = Group::new();
        g1.set_transform(rotation_y(std::f64::consts::FRAC_PI_2));
    
        let mut g2 = Group::new();
        g2.set_transform(scaling(2.0, 2.0, 2.0));
    
        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));
    
        s.shape_data_mut().parent = Some(g2.id());
        g2.add_child(&s);
    
        g2.shape_data_mut().parent = Some(g1.id());
        g1.add_child(&g2);
    
        let resolve = |id: u64| -> Option<&dyn Shape> {
            if id == g1.id() {
                Some(&g1 as &dyn Shape)
            } else if id == g2.id() {
                Some(&g2 as &dyn Shape)
            } else if id == s.id() {
                Some(&s as &dyn Shape)
            } else {
                None
            }
        };
    
        let p = world_to_object(&s, &resolve, &Tuple::point(-2.0, 0.0, -10.0));
    
        assert!(p.is_equal(&Tuple::point(0.0, 0.0, -1.0)));
    }

    #[test]
    fn converting_a_normal_from_object_to_world_space() {
        use crate::shape::normal_to_world;
        use crate::sphere::Sphere;
        use crate::transformation::{rotation_y, scaling, translation};
        use crate::group::Group;
        use std::f64::consts::FRAC_PI_2;
    
        let mut g1 = Group::new();
        g1.set_transform(rotation_y(FRAC_PI_2));
    
        let mut g2 = Group::new();
        g2.set_transform(scaling(1.0, 2.0, 3.0));
    
        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));
    
        s.shape_data_mut().parent = Some(g2.id());
        g2.add_child(&s);
    
        g2.shape_data_mut().parent = Some(g1.id());
        g1.add_child(&g2);
    
        let resolve = |id: u64| -> Option<&dyn Shape> {
            if id == g1.id() {
                Some(&g1 as &dyn Shape)
            } else if id == g2.id() {
                Some(&g2 as &dyn Shape)
            } else if id == s.id() {
                Some(&s as &dyn Shape)
            } else {
                None
            }
        };
    
        let n_obj = Tuple::vector(
            3.0f64.sqrt() / 3.0,
            3.0f64.sqrt() / 3.0,
            3.0f64.sqrt() / 3.0,
        );
        let n = normal_to_world(&s, &resolve, &n_obj);
        assert!(n.is_equal(&Tuple::vector(0.28571, 0.42857, -0.85714)));
    }

    #[test]
    fn finding_the_normal_on_a_child_object() {
        use crate::group::Group;
        use crate::shape::{shape_normal_at, Shape};
        use crate::sphere::Sphere;
        use crate::transformation::{rotation_y, scaling, translation};
        use std::f64::consts::FRAC_PI_2;
    
        let mut g1 = Group::new();
        g1.set_transform(rotation_y(FRAC_PI_2));
    
        let mut g2 = Group::new();
        g2.set_transform(scaling(1.0, 2.0, 3.0));
    
        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));
    
        s.shape_data_mut().parent = Some(g2.id());
        g2.add_child(&s);
    
        g2.shape_data_mut().parent = Some(g1.id());
        g1.add_child(&g2);
    
        let resolve = |id: u64| -> Option<&dyn Shape> {
            if id == g1.id() {
                Some(&g1 as &dyn Shape)
            } else if id == g2.id() {
                Some(&g2 as &dyn Shape)
            } else if id == s.id() {
                Some(&s as &dyn Shape)
            } else {
                None
            }
        };
    
        let p = Tuple::point(1.7321, 1.1547, -5.5774);
        let n = shape_normal_at(&s, &resolve, &p);
        
        assert!(n.is_equal(&Tuple::vector(0.285703, 0.4285431, -0.857160)));
    }
}

// TODO : Look at the world_to_object and normal_to_world function again and understand the syntax there