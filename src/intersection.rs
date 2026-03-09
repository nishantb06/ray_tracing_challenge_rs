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
}
