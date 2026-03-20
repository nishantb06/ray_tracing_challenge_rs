use crate::canvas::Color;
use crate::light::PointLight;
use crate::pattern::{Pattern};  
use crate::shape::Shape;
use crate::tuple::Tuple;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub pattern: Option<Box<dyn Pattern>>,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
}

#[allow(dead_code)]
impl Material {
    pub fn new() -> Self {
        Material {
            color: Color::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            pattern: None,
            reflective: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.color.is_equal(&other.color)
            && self.ambient == other.ambient
            && self.diffuse == other.diffuse
            && self.specular == other.specular
            && self.shininess == other.shininess
            // optionally ignore pattern, or treat None/Some differently
    }
}

pub fn lighting(
    m: &Material,
    object: &dyn Shape,
    light: &PointLight,
    point: &Tuple,
    eye_vector: &Tuple,
    normal_vector: &Tuple,
    in_shadow: bool,
) -> Color {
    // combine the surface color with the light's color/intensity
    let effective_color = match &m.pattern {
        Some(pattern) => {
            let pattern_color =
                pattern.pattern_at_shape(object, &point);
            &pattern_color * &light.intensity
        }
        None => &m.color * &light.intensity,
    };

    // find the direction to the light source
    let mut light_v = &light.position - point;
    light_v = light_v.normalize();

    // compute the ambient contribution
    let ambient = &effective_color * m.ambient;
    let diffuse;
    let specular;
    // light_dot_normal represents the cosine of the angle between the
    // light vector and the normal vector. A negative number means the
    // light is on the other side of the surface.
    let light_dot_normal = light_v.dot(normal_vector);
    if light_dot_normal < 0.0 || in_shadow {
        diffuse = Color::new(0.0, 0.0, 0.0);
        specular = Color::new(0.0, 0.0, 0.0);
    } else {
        diffuse = &effective_color * (m.diffuse * light_dot_normal);

        // reflect_dot_eye represents the cosine of the angle between the
        // reflection vector and the eye vector. A negative number means the
        // light reflects away from the eye.
        let reflect_v = -&light_v.reflect(normal_vector);
        let reflect_dot_eye = reflect_v.dot(eye_vector);
        if reflect_dot_eye <= 0.0 {
            specular = Color::new(0.0, 0.0, 0.0);
        } else {
            let factor = reflect_dot_eye.powf(m.shininess);
            specular = &light.intensity * (m.specular * factor);
        }
    }
    return &(&ambient + &diffuse) + &specular;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{BLACK, WHITE};
    use crate::sphere::Sphere;
    use crate::utils::equal;
    use crate::pattern::StripePattern;

    #[test]
    fn the_default_material() {
        let m = Material::new();
        assert!(m.color.is_equal(&Color::new(1.0, 1.0, 1.0)));
        assert!(equal(m.ambient, 0.1));
        assert!(equal(m.diffuse, 0.9));
        assert!(equal(m.specular, 0.9));
        assert!(equal(m.shininess, 200.0));
    }

    #[test]
    fn lighting_with_eye_between_light_and_surface() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, false);
        assert!(result.is_equal(&Color::new(1.9, 1.9, 1.9)));
    }

    #[test]
    fn lighting_with_eye_between_light_and_surface_eye_offset_45() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        );
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, false);
        assert!(result.is_equal(&Color::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, false);
        assert!(result.is_equal(&Color::new(0.7364, 0.7364, 0.7364)));
    }

    #[test]
    fn lighting_with_eye_in_path_of_reflection_vector() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(
            0.0,
            -std::f64::consts::FRAC_1_SQRT_2,
            -std::f64::consts::FRAC_1_SQRT_2,
        );
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, false);
        assert!(result.is_equal(&Color::new(1.6364, 1.6364, 1.6364)));
    }

    #[test]
    fn lighting_with_light_behind_surface() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, false);
        assert!(result.is_equal(&Color::new(0.1, 0.1, 0.1)));
    }

    #[test]
    fn lighting_with_the_surface_in_shadow() {
        let m = Material::new();
        let object = Sphere::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = true;
        let result = lighting(&m, &object, &light, &position, &eyev, &normalv, in_shadow);
        assert!(result.is_equal(&Color::new(0.1, 0.1, 0.1)));
    }

    #[test]
    fn lighting_with_a_pattern_applied() {
        let m = Material {
            color: Color::new(1.0, 1.0, 1.0),
            ambient: 1.0,
            diffuse: 0.0,
            specular: 0.0,
            shininess: 200.0,
            pattern: Some(Box::new(StripePattern::new(WHITE, BLACK))),
            reflective:0.0,
            transparency:0.0,
            refractive_index:1.0,
        };
        let object = Sphere::new();

        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(
            Tuple::point(0.0, 0.0, -10.0),
            Color::new(1.0, 1.0, 1.0),
        );

        let c1 = lighting(
            &m,
            &object,
            &light,
            &Tuple::point(0.9, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        let c2 = lighting(
            &m,
            &object,
            &light,
            &Tuple::point(1.1, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );

        assert!(c1.is_equal(&Color::new(1.0, 1.0, 1.0)));
        assert!(c2.is_equal(&Color::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn default_material_transparency_and_refractive_index() {
        let m = Material::new();
        assert!(equal(m.transparency, 0.0));
        assert!(equal(m.refractive_index, 1.0));
    }
}

// TODO : Also, you may soon realize that materials applied to a group have no effect
// at all on the shapes it contains. What if you wanted the shapes in your ray
// tracer to be able to “inherit” materials from their parents? How might you
// extend your code to make that happen?