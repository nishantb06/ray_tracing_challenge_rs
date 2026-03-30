use std::collections::HashSet;

use crate::intersection::Intersections;
use crate::ray::Ray;
use crate::shape::{Shape, ShapeData};
use crate::tuple::Tuple;


#[derive(Debug)]
pub struct Group {
    pub data: ShapeData,
    pub shapes: Vec<Box<dyn Shape>>, // TODO: we can make this a HashSet as well
    pub ids: HashSet<u64>,
}


impl Group {
    pub fn new() -> Self {
        Group {
            data: ShapeData::new(),
            shapes: vec![],
            ids: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    pub fn add_child(&mut self, mut shape: Box<dyn Shape>) {
        let parent_id = self.id(); // avoid borrow conflicts
    
        shape.shape_data_mut().parent = Some(parent_id);
        self.ids.insert(shape.id());
        self.shapes.push(shape);
    }

    pub fn includes(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }
}

impl Shape for Group {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    // ray intersects a group if and only if the ray intersects at least one child shape contained by the group.
    fn local_intersect<'b>(&'b self, ray: &Ray) -> Intersections<'b> {
        let mut all_intersections = Vec::new();
        for shape in &self.shapes {
            let xs = shape.intersect(ray);
            all_intersections.extend(xs.data);
        }
        Intersections::new(all_intersections)
    }

    fn local_normal_at(&self, _local_point: &Tuple) -> Tuple {
        panic!(
            "Group has no surface: intersections should use the leaf shape; \
             use shape_normal_at(leaf, resolve, world_point) instead"
        );
    }

    fn find_by_id(&self, id: u64) -> Option<&dyn Shape> {
        if self.id() == id {
            return Some(self);
        }
    
        for shape in &self.shapes {
            // Many leaf shapes (e.g. `Sphere`) use the default `find_by_id` (returns None), so we must match by id directly here.
            if shape.id() == id {
                return Some(shape.as_ref());
            }
    
            // If it's a nested Group, its overridden `find_by_id` can recurse.
            if let Some(found) = shape.find_by_id(id) {
                return Some(found);
            }
        }
    
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::test_support::TestShape;

    fn test_shape() -> TestShape {
        TestShape::new()
    }

    #[test]
    fn group_is_a_shape() {
        fn assert_is_shape<T: Shape>(_: &T) {}
        let c = Group::new();
        assert_is_shape(&c);
    }

    #[test]
    fn creating_a_new_group() {
        let g = Group::new();
        assert_eq!(g.len(),0);
    }

    #[test]
    fn adding_a_child_to_the_group() {
        let mut g = Group::new();
        let s = test_shape();
        let s_id = s.id();
        g.add_child(Box::new(s));
        assert!(g.includes(s_id));
    }

    #[test]
    fn intersecting_ray_with_empty_group() {
        let g = Group::new();
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
        assert!(xs.data.is_empty());
    }

    #[test]
    fn intersecting_ray_with_nonempty_group() {
        use crate::sphere::Sphere;
        use crate::transformation::translation;
    
        let s1 = Sphere::new();
        let s1_id = s1.id();
        let mut s2 = Sphere::new();
        s2.set_transform(translation(0.0, 0.0, -3.0));
        let s2_id = s2.id();
        let mut s3 = Sphere::new();
        s3.set_transform(translation(5.0, 0.0, 0.0));
    
        let mut g = Group::new();
        g.add_child(Box::new(s1));
        g.add_child(Box::new(s2));
        g.add_child(Box::new(s3));
    
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
    
        assert_eq!(xs.count(), 4);
        assert_eq!(xs.data[0].object.id(), s2_id);
        assert_eq!(xs.data[1].object.id(), s2_id);
        assert_eq!(xs.data[2].object.id(), s1_id);
        assert_eq!(xs.data[3].object.id(), s1_id);
    }

    #[test]
    fn intersecting_transformed_group() {
        use crate::sphere::Sphere;
        use crate::transformation::{scaling, translation};
    
        let mut g = Group::new();
        g.set_transform(scaling(2.0, 2.0, 2.0));
    
        let mut s = Sphere::new();
        s.set_transform(translation(5.0, 0.0, 0.0));
    
        g.add_child(Box::new(s));
    
        let r = Ray::new(Tuple::point(10.0, 0.0, -10.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.intersect(&r);
    
        assert_eq!(xs.count(), 2);
    }

    #[test]
    fn transformed_group_intersects_leaked_triangle_children() {
        use crate::transformation::{rotation_y, translation};
        use crate::triangle::Triangle;

        let mut g = Group::new();
        g.set_transform(&rotation_y(0.35) * &translation(0.0, 0.0, 0.25));
        
        let mut tri = Box::new(Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        ));
        
        tri.shape_data_mut().parent = Some(g.id());
        g.add_child(tri);
        
        let r = Ray::new(Tuple::point(0.0, 0.5, -4.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.intersect(&r);
        assert!(xs.count() >= 1, "expected at least one hit through group + triangle");
    }
}

// TODO: Go read about lifetimes and see why they are important here , for example why does the local_intersect have a different lifetime param?
// TODO: Read about leaked Box pointers why they are important here ?
