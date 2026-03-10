use crate::canvas::Color;

#[derive(Debug)]
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
}
