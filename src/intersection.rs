use crate::sphere::Sphere;

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
}
