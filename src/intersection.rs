use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::tuple::Tuple;
use crate::utils::EPSILON;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a Sphere,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Intersections<'a> {
    pub data: Vec<Intersection<'a>>,
}

#[allow(dead_code)]
impl<'a> Intersection<'a> {
    pub fn new(t: f64, object: &'a Sphere) -> Self {
        Intersection { t, object }
    }
}

pub struct Computations<'a> {
    pub t: f64,
    pub object: &'a Sphere,
    pub point: Tuple,
    pub eye_vector: Tuple,
    pub normal_vector: Tuple,
    pub inside: bool,
    pub over_point: Tuple,
}

// precomputes the point (in world space) where the intersection occurred,
// the eye vector (pointing back toward the eye, or camera), and the normal vector.
pub fn prepare_computations<'a>(intersection: &'a Intersection<'a>, ray: &Ray) -> Computations<'a> {
    let point = ray.position(intersection.t.clone());
    let mut normal_v = intersection.object.normal_at(&point);
    let eye_v = -&(ray.direction);
    let mut inside = false;
    if normal_v.dot(&eye_v) < 0.0 {
        inside = true;
        normal_v = -&normal_v;
    }
    let over_point = &point + &(&normal_v * EPSILON);
    Computations {
        t: intersection.t.clone(),
        object: intersection.object,
        point: point,
        eye_vector: eye_v,
        normal_vector: normal_v,
        inside,
        over_point,
    }
}

#[allow(dead_code)]
impl<'a> Intersections<'a> {
    pub fn new(mut items: Vec<Intersection<'a>>) -> Self {
        items.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        Intersections { data: items }
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn hit(&self) -> Option<&Intersection<'a>> {
        self.data.iter().find(|i| i.t >= 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_intersection_encapsulates_t_and_object() {
        let s = Sphere::new();
        let i = Intersection::new(3.5, &s);
        assert!(crate::utils::equal(i.t, 3.5));
        assert_eq!(i.object.id, s.id);
    }

    #[test]
    fn aggregating_intersections() {
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i1, i2]);
        assert_eq!(xs.count(), 2);
        assert!(crate::utils::equal(xs.data[0].t, 1.0));
        assert!(crate::utils::equal(xs.data[1].t, 2.0));
    }

    #[test]
    fn the_hit_when_all_intersections_have_positive_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 1.0));
    }

    #[test]
    fn the_hit_when_some_intersections_have_negative_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(1.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 1.0));
    }

    #[test]
    fn the_hit_when_all_intersections_have_negative_t() {
        let s = Sphere::new();
        let i1 = Intersection::new(-2.0, &s);
        let i2 = Intersection::new(-1.0, &s);
        let xs = Intersections::new(vec![i2, i1]);
        let i = xs.hit();
        assert!(i.is_none());
    }

    #[test]
    fn the_hit_is_always_the_lowest_nonnegative_intersection() {
        let s = Sphere::new();
        let i1 = Intersection::new(5.0, &s);
        let i2 = Intersection::new(7.0, &s);
        let i3 = Intersection::new(-3.0, &s);
        let i4 = Intersection::new(2.0, &s);
        let xs = Intersections::new(vec![i1, i2, i3, i4]);
        let i = xs.hit().unwrap();
        assert!(crate::utils::equal(i.t, 2.0));
    }

    #[test]
    fn precomputing_the_state_of_an_intersection() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(4.0, &shape);
        let comps = prepare_computations(&i, &r);
        assert!(crate::utils::equal(comps.t, i.t));
        assert_eq!(comps.object.id, i.object.id);
        assert!(comps.point.is_equal(&Tuple::point(0.0, 0.0, -1.0)));
        assert!(comps.eye_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
        assert!(comps.normal_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
    }

    #[test]
    fn the_hit_when_intersection_occurs_on_the_outside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(4.0, &shape);
        let comps = prepare_computations(&i, &r);
        assert!(!comps.inside);
    }

    #[test]
    fn the_hit_when_intersection_occurs_on_the_inside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(1.0, &shape);
        let comps = prepare_computations(&i, &r);
        assert!(comps.point.is_equal(&Tuple::point(0.0, 0.0, 1.0)));
        assert!(comps.eye_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
        assert!(comps.inside);
        assert!(comps.normal_vector.is_equal(&Tuple::vector(0.0, 0.0, -1.0)));
    }
}
