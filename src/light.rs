use crate::tuple::Tuple;
use crate::canvas::Color;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PointLight {
    pub position: Tuple,
    pub intensity: Color,
}

#[allow(dead_code)]
impl PointLight {
    pub fn new(position: Tuple, intensity: Color) -> Self {
        PointLight { position, intensity }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_light_has_a_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Tuple::point(0.0, 0.0, 0.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0));
        assert!(light.position.is_equal(&position));
        assert!(light.intensity.is_equal(&intensity));
    }
}
