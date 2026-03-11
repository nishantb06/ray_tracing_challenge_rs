use crate::canvas::Color;
use crate::tuple::Tuple;
use crate::light::PointLight;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
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
        }
    }
}

pub fn lighting(
        m: &Material,
        light: &PointLight,
        point : &Tuple,
        eye_vector : &Tuple,
        normal_vector : &Tuple
    ) -> Color {
    // combine the surface color with the light's color/intensity
    let effective_color = &m.color * &light.intensity;
    
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
    if light_dot_normal < 0.0 {
        diffuse = Color::new(0.0, 0.0, 0.0);
        specular = Color::new(0.0, 0.0, 0.0);
    } else {
        diffuse = &effective_color * (m.diffuse * light_dot_normal);
        
        // reflect_dot_eye represents the cosine of the angle between the
        // reflection vector and the eye vector. A negative number means the
        // light reflects away from the eye.
        let reflect_v  = -&light_v.reflect(normal_vector);
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
    use crate::utils::equal;

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
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &light, &position, &eyev, &normalv);
        assert!(result.is_equal(&Color::new(1.9, 1.9, 1.9)));
    }
    
    #[test]
    fn lighting_with_eye_between_light_and_surface_eye_offset_45() {
        let m = Material::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &light, &position, &eyev, &normalv);
        assert!(result.is_equal(&Color::new(1.0, 1.0, 1.0)));
    }
    
    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45() {
        let m = Material::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &light, &position, &eyev, &normalv);
        assert!(result.is_equal(&Color::new(0.7364, 0.7364, 0.7364)));
    }
    
    #[test]
    fn lighting_with_eye_in_path_of_reflection_vector() {
        let m = Material::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, -std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &light, &position, &eyev, &normalv);
        assert!(result.is_equal(&Color::new(1.6364, 1.6364, 1.6364)));
    }
    
    #[test]
    fn lighting_with_light_behind_surface() {
        let m = Material::new();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = lighting(&m, &light, &position, &eyev, &normalv);
        assert!(result.is_equal(&Color::new(0.1, 0.1, 0.1)));
    }
}
