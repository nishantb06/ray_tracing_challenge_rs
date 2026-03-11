use crate::sphere::Sphere;
use crate::light::PointLight;
use crate::tuple::Tuple;
use crate::canvas::Color;
use crate::transformation::scaling;
use crate::intersection::{Intersection, Intersections, Computations, prepare_computations};
use crate::ray::Ray;
use crate::material::lighting;

#[derive(Debug)]
#[allow(dead_code)]
pub struct World {
    pub objects: Vec<Sphere>,
    pub lights: Vec<PointLight>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        World {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn default_world() -> Self {
        let light = PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        );

        let mut s1 = Sphere::new();
        s1.material.color = Color::new(0.8, 1.0, 0.6);
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;

        let mut s2 = Sphere::new();
        s2.set_transform(scaling(0.5, 0.5, 0.5));

        World {
            objects: vec![s1, s2],
            lights: vec![light],
        }
    }

    // checks if the world contains a sphere with matching material properties and transform 
    // (compared by value, not by ID, since each Sphere::new() generates a unique ID
    pub fn contains(&self, sphere: &Sphere) -> bool {
        self.objects.iter().any(|o| {
            o.material.color.is_equal(&sphere.material.color)
                && crate::utils::equal(o.material.ambient, sphere.material.ambient)
                && crate::utils::equal(o.material.diffuse, sphere.material.diffuse)
                && crate::utils::equal(o.material.specular, sphere.material.specular)
                && crate::utils::equal(o.material.shininess, sphere.material.shininess)
                && o.transform == sphere.transform
        })
    }

    pub fn intersect_world(&self, ray: &Ray) -> Intersections<'_> {
        let mut all: Vec<Intersection> = Vec::new();
        for obj in &self.objects {
            let obj_xs = obj.intersect(ray);
            all.extend(obj_xs.data);
        }
        let xs = Intersections::new(all);
        return xs;
    }
}

// returns the color at the intersection encapsulated by comps, in the given world.
// iterate over all lights and sum the colors
pub fn shade_hit(world: &World, comps: &Computations) -> Color {
    world.lights.iter().fold(Color::new(0.0, 0.0, 0.0), |acc, light| {
        let c = lighting(
            &comps.object.material,
            light,
            &comps.point,
            &comps.eye_vector,
            &comps.normal_vector,
        );
        &acc + &c
    })
}

// It will intersect the world with the given ray and then return the color at the resulting intersection.
pub fn color_at(world: &World, ray: &Ray) -> Color {
    // find all the points where the ray would intersect the world
    let xs: Intersections = world.intersect_world(ray);

    // out of all the intersections find "the hit" intersection with the lowest positive t
    let hit = xs.hit();
    let comps;
    if hit.is_none() {
        return Color::new(0.0,0.0,0.0);
    } else {
        // at that specific intersection, calculate the necessary data to get the color
        comps = prepare_computations(
            &hit.unwrap(),
            ray,
        );
        // return the result of the function which calculates the color with the help of the precomputed data in the above step
        return shade_hit(world,&comps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray::Ray;

    #[test]
    fn creating_a_world() {
        let w = World::new();
        assert_eq!(w.objects.len(), 0);
        assert!(w.lights.is_empty());
    }

    #[test]
    fn the_default_world() {
        let light = PointLight::new(
            Tuple::point(-10.0, 10.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        );

        let mut s1 = Sphere::new();
        s1.material.color = Color::new(0.8, 1.0, 0.6);
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;

        let mut s2 = Sphere::new();
        s2.set_transform(scaling(0.5, 0.5, 0.5));

        let w = World::default_world();

        assert_eq!(w.lights.len(), 1);
        assert!(w.lights[0].position.is_equal(&light.position));
        assert!(w.lights[0].intensity.is_equal(&light.intensity));
        assert!(w.contains(&s1));
        assert!(w.contains(&s2));
    }

    #[test]
    fn intersect_a_world_with_a_ray() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = w.intersect_world(&r);
        assert_eq!(xs.count(), 4);
        assert!(crate::utils::equal(xs.data[0].t, 4.0));
        assert!(crate::utils::equal(xs.data[1].t, 4.5));
        assert!(crate::utils::equal(xs.data[2].t, 5.5));
        assert!(crate::utils::equal(xs.data[3].t, 6.0));
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[0];
        let i = Intersection::new(4.0, shape);
        let comps = crate::intersection::prepare_computations(&i, &r);
        let c = shade_hit(&w, &comps);
        assert!(c.is_equal(&Color::new(0.38066, 0.47583, 0.2855)));
    }
    
    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default_world();
        w.lights = vec![PointLight::new(
            Tuple::point(0.0, 0.25, 0.0),
            Color::new(1.0, 1.0, 1.0),
        )];
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[1];
        let i = Intersection::new(0.5, shape);
        let comps = crate::intersection::prepare_computations(&i, &r);
        let c = shade_hit(&w, &comps);
        assert!(c.is_equal(&Color::new(0.90498, 0.90498, 0.90498)));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = color_at(&w, &r);
        assert!(c.is_equal(&Color::new(0.0, 0.0, 0.0)));
    }
    
    #[test]
    fn the_color_when_a_ray_hits() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = color_at(&w, &r);
        assert!(c.is_equal(&Color::new(0.38066, 0.47583, 0.2855)));
    }
    
    #[test]
    fn the_color_with_an_intersection_behind_the_ray() {
        let mut w = World::default_world();
        w.objects[0].material.ambient = 1.0;
        w.objects[1].material.ambient = 1.0;
        let inner_color = w.objects[1].material.color.clone();
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let c = color_at(&w, &r);
        assert!(c.is_equal(&inner_color));
    }
}
