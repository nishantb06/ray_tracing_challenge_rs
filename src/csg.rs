use crate::shape::{Shape, ShapeData};
use crate::intersection::{Intersection, Intersections};
use crate::ray::Ray;
use crate::tuple::Tuple;

#[derive(Debug,Copy,Clone)]
pub enum CSGOperation {
    Union,
    Intersection,
    Difference,
}

#[derive(Debug)]
pub struct CSG {
    pub data: ShapeData,
    pub operation: CSGOperation,
    pub left: Box<dyn Shape>,
    pub right: Box<dyn Shape>,
}



impl Shape for CSG {
    fn shape_data(&self) -> &ShapeData {
        &self.data
    }

    fn shape_data_mut(&mut self) -> &mut ShapeData {
        &mut self.data
    }

    fn local_intersect<'b>(&'b self, ray: &Ray) -> Intersections<'b> {
        let mut xs = self.left.intersect(ray).data;
        xs.extend(self.right.intersect(ray).data);
    
        let xs = Intersections::new(xs);          // sorts by t
        self.filter_intersections(&xs)            // returns filtered intersections
    }

    fn local_normal_at(&self, _local_point: &Tuple, _hit: Option<&Intersection>) -> Tuple {
        panic!("CSG has no surface: normals should be computed on the leaf shape that was hit");
    }

    fn find_by_id(&self, id: u64) -> Option<&dyn Shape> {
        if self.id() == id {
            return Some(self);
        }

        // direct children
        if self.left.id() == id {
            return Some(self.left.as_ref());
        }
        if self.right.id() == id {
            return Some(self.right.as_ref());
        }

        // recurse into subtrees (Group/CSG can override find_by_id)
        if let Some(found) = self.left.find_by_id(id) {
            return Some(found);
        }
        if let Some(found) = self.right.find_by_id(id) {
            return Some(found);
        }

        None
    }
}

impl CSG {
    pub fn new(
        operation: CSGOperation,
        mut s1: Box<dyn Shape>,
        mut s2: Box<dyn Shape>
    ) -> Self {
        let data = ShapeData::new();
        let parent_id = data.id;
        
        s1.shape_data_mut().parent = Some(parent_id);
        s2.shape_data_mut().parent = Some(parent_id);

        CSG {
            data,
            operation,
            left: s1,
            right: s2,
        }
    }

    pub fn intersection_allowed_for(op: CSGOperation, lhit: bool, inl: bool, inr: bool) -> bool {
        match op {
            CSGOperation::Union =>
                (lhit && !inr) || (!lhit && !inl),
            CSGOperation::Intersection =>
                (lhit && inr) || (!lhit && inl),
            CSGOperation::Difference =>
                (lhit && !inr) || (!lhit && inl),
        }
    }
    pub fn intersection_allowed(&self, lhit: bool, inl: bool, inr: bool) -> bool {
        Self::intersection_allowed_for(self.operation, lhit, inl, inr)
    }

    fn contains(root: &dyn Shape, id: u64) -> bool {
        // Works for leaf shapes (id match) and for Groups (via overridden find_by_id)
        root.id() == id || root.find_by_id(id).is_some()
    }

    pub fn filter_intersections<'a>(&self, xs: &Intersections<'a>) -> Intersections<'a> {
        let mut inl = false;
        let mut inr = false;
        let mut kept: Vec<Intersection<'a>> = Vec::new();
        for i in &xs.data {
            let lhit = Self::contains(self.left.as_ref(), i.object.id());
            // Use whichever API you kept:
            // - if you kept the static one:
            let allowed = Self::intersection_allowed_for(self.operation, lhit, inl, inr);
            // - or if you prefer the &self one:
            // let allowed = self.intersection_allowed(lhit, inl, inr);
            if allowed {
                kept.push(i.clone());
            }
            // advance inside/outside state
            if lhit {
                inl = !inl;
            } else {
                inr = !inr;
            }
        }
        Intersections::new(kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::Cube;
    use crate::shape::Shape;
    use crate::sphere::Sphere;

    #[derive(Debug, Clone, Copy)]
    struct Case {
        op: CSGOperation,
        lhit: bool,
        inl: bool,
        inr: bool,
        result: bool,
    }

    #[test]
    fn creating_a_csg_sets_operation_children_and_parent_ids() {
        let s1 = Sphere::new();
        let s1_id = s1.id();

        let s2 = Cube::new();
        let s2_id = s2.id();

        let c = CSG::new(CSGOperation::Union, Box::new(s1), Box::new(s2));

        assert!(matches!(c.operation, CSGOperation::Union));

        assert_eq!(c.left.id(), s1_id);
        assert_eq!(c.right.id(), s2_id);

        assert_eq!(c.left.shape_data().parent, Some(c.id()));
        assert_eq!(c.right.shape_data().parent, Some(c.id()));
    }

    #[test]
    fn evaluating_the_rule_for_a_csg_operation() {
        let cases = [
            // Union
            Case { op: CSGOperation::Union, lhit: true,  inl: true,  inr: true,  result: false },
            Case { op: CSGOperation::Union, lhit: true,  inl: true,  inr: false, result: true  },
            Case { op: CSGOperation::Union, lhit: true,  inl: false, inr: true,  result: false },
            Case { op: CSGOperation::Union, lhit: true,  inl: false, inr: false, result: true },
            Case { op: CSGOperation::Union, lhit: false, inl: true,  inr: true,  result: false },
            Case { op: CSGOperation::Union, lhit: false, inl: true,  inr: false, result: false },
            Case { op: CSGOperation::Union, lhit: false, inl: false, inr: true,  result: true  },
            Case { op: CSGOperation::Union, lhit: false, inl: false, inr: false, result: true  },
            // Intersection
            Case { op: CSGOperation::Intersection, lhit: true,  inl: true,  inr: true,  result: true  },
            Case { op: CSGOperation::Intersection, lhit: true,  inl: true,  inr: false, result: false },
            Case { op: CSGOperation::Intersection, lhit: true,  inl: false, inr: true,  result: true  },
            Case { op: CSGOperation::Intersection, lhit: true,  inl: false, inr: false, result: false },
            Case { op: CSGOperation::Intersection, lhit: false, inl: true,  inr: true,  result: true  },
            Case { op: CSGOperation::Intersection, lhit: false, inl: true,  inr: false, result: true  },
            Case { op: CSGOperation::Intersection, lhit: false, inl: false, inr: true,  result: false },
            Case { op: CSGOperation::Intersection, lhit: false, inl: false, inr: false, result: false },
            // Difference
            Case { op: CSGOperation::Difference, lhit: true,  inl: true,  inr: true,  result: false },
            Case { op: CSGOperation::Difference, lhit: true,  inl: true,  inr: false, result: true  },
            Case { op: CSGOperation::Difference, lhit: true,  inl: false, inr: true,  result: false },
            Case { op: CSGOperation::Difference, lhit: true,  inl: false, inr: false, result: true },
            Case { op: CSGOperation::Difference, lhit: false, inl: true,  inr: true,  result: true  },
            Case { op: CSGOperation::Difference, lhit: false, inl: true,  inr: false, result: true  },
            Case { op: CSGOperation::Difference, lhit: false, inl: false, inr: true,  result: false },
            Case { op: CSGOperation::Difference, lhit: false, inl: false, inr: false, result: false },
        ];
        for (i, tc) in cases.iter().enumerate() {
            let got = CSG::intersection_allowed_for(tc.op, tc.lhit, tc.inl, tc.inr);
            assert_eq!(got, tc.result, "case {i:?}: {tc:?}");
        }
        for (i, tc) in cases.iter().enumerate() {
            let got_static = CSG::intersection_allowed_for(tc.op, tc.lhit, tc.inl, tc.inr);
        
            // make an instance so we can call the &self version too
            let csg = CSG::new(tc.op, Box::new(Sphere::new()), Box::new(Cube::new()));
            let got_method = csg.intersection_allowed(tc.lhit, tc.inl, tc.inr);
        
            assert_eq!(got_static, tc.result, "static case {i:?}: {tc:?}");
            assert_eq!(got_method, tc.result, "method case {i:?}: {tc:?}");
        }
    }

    #[test]
    fn filtering_a_list_of_intersections() {
        #[derive(Debug, Clone, Copy)]
        struct Case {
            operation: CSGOperation,
            x0: usize,
            x1: usize,
        }
        let cases = [
            Case { operation: CSGOperation::Union,        x0: 0, x1: 3 },
            Case { operation: CSGOperation::Intersection, x0: 1, x1: 2 },
            Case { operation: CSGOperation::Difference,   x0: 0, x1: 1 },
        ];
        for (i, tc) in cases.iter().enumerate() {
            let c = CSG::new(tc.operation, Box::new(Sphere::new()), Box::new(Cube::new()));
            // xs ← intersections(1:s1, 2:s2, 3:s1, 4:s2)
            // Here s1/s2 are c.left/c.right
            let s1 = c.left.as_ref();
            let s2 = c.right.as_ref();
            let xs = Intersections::new(vec![
                Intersection::new(1.0, s1),
                Intersection::new(2.0, s2),
                Intersection::new(3.0, s1),
                Intersection::new(4.0, s2),
            ]);
            let result = c.filter_intersections(&xs);
            assert_eq!(result.count(), 2, "case {i}: {tc:?}");
            // result[0] = xs[x0]
            assert!(crate::utils::equal(result.data[0].t, xs.data[tc.x0].t), "case {i}: {tc:?}");
            assert_eq!(result.data[0].object.id(), xs.data[tc.x0].object.id(), "case {i}: {tc:?}");
            // result[1] = xs[x1]
            assert!(crate::utils::equal(result.data[1].t, xs.data[tc.x1].t), "case {i}: {tc:?}");
            assert_eq!(result.data[1].object.id(), xs.data[tc.x1].object.id(), "case {i}: {tc:?}");
        }
    }

    #[test]
    fn a_ray_misses_a_csg_object() {
        use crate::cube::Cube;
        use crate::sphere::Sphere;
    
        let c = CSG::new(CSGOperation::Union, Box::new(Sphere::new()), Box::new(Cube::new()));
        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    
        let xs = c.local_intersect(&r);
        assert!(xs.data.is_empty());
    }
    
    #[test]
    fn a_ray_hits_a_csg_object() {
        use crate::sphere::Sphere;
        use crate::transformation::translation;
    
        let s1 = Sphere::new();
        let s1_id = s1.id();
    
        let mut s2 = Sphere::new();
        s2.set_transform(translation(0.0, 0.0, 0.5));
        let s2_id = s2.id();
    
        let c = CSG::new(CSGOperation::Union, Box::new(s1), Box::new(s2));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    
        let xs = c.local_intersect(&r);
    
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 4.0));
        assert_eq!(xs.data[0].object.id(), s1_id);
    
        assert!(crate::utils::equal(xs.data[1].t, 6.5));
        assert_eq!(xs.data[1].object.id(), s2_id);
    }
}