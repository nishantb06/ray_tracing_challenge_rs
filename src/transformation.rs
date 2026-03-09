use crate::matrix::Matrix;

#[allow(dead_code)]
pub fn translation(x: f64, y: f64, z: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[0][3] = x;
    m.data[1][3] = y;
    m.data[2][3] = z;
    m
}

#[allow(dead_code)]
pub fn scaling(x: f64, y: f64, z: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[0][0] = x;
    m.data[1][1] = y;
    m.data[2][2] = z;
    m
}

#[allow(dead_code)]
pub fn rotation_x(r: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[1][1] = r.cos();
    m.data[1][2] = -r.sin();
    m.data[2][1] = r.sin();
    m.data[2][2] = r.cos();
    m
}

#[allow(dead_code)]
pub fn rotation_y(r: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[0][0] = r.cos();
    m.data[0][2] = r.sin();
    m.data[2][0] = -r.sin();
    m.data[2][2] = r.cos();
    m
}

#[allow(dead_code)]
pub fn rotation_z(r: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[0][0] = r.cos();
    m.data[0][1] = -r.sin();
    m.data[1][0] = r.sin();
    m.data[1][1] = r.cos();
    m
}

#[allow(dead_code)]
pub fn shearing(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix {
    let mut m = Matrix::identity(4);
    m.data[0][1] = xy;
    m.data[0][2] = xz;
    m.data[1][0] = yx;
    m.data[1][2] = yz;
    m.data[2][0] = zx;
    m.data[2][1] = zy;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuple::Tuple;

    #[test]
    fn multiplying_by_translation_matrix() {
        let transform = translation(5.0, -3.0, 2.0);
        let p = Tuple::point(-3.0, 4.0, 5.0);
        let expected = Tuple::point(2.0, 1.0, 7.0);
        assert!(&transform * &p == expected);
    }

    #[test]
    fn multiplying_by_inverse_of_translation_matrix() {
        let transform = translation(5.0, -3.0, 2.0);
        let inv = transform.inverse_gauss_jordan();
        let p = Tuple::point(-3.0, 4.0, 5.0);
        let expected = Tuple::point(-8.0, 7.0, 3.0);
        assert!(&inv * &p == expected);
    }

    #[test]
    fn translation_does_not_affect_vectors() {
        let transform = translation(5.0, -3.0, 2.0);
        let v = Tuple::vector(-3.0, 4.0, 5.0);
        assert!(&transform * &v == v);
    }

    #[test]
    fn scaling_matrix_applied_to_point() {
        let transform = scaling(2.0, 3.0, 4.0);
        let p = Tuple::point(-4.0, 6.0, 8.0);
        assert!(&transform * &p == Tuple::point(-8.0, 18.0, 32.0));
    }

    #[test]
    fn scaling_matrix_applied_to_vector() {
        let transform = scaling(2.0, 3.0, 4.0);
        let v = Tuple::vector(-4.0, 6.0, 8.0);
        assert!(&transform * &v == Tuple::vector(-8.0, 18.0, 32.0));
    }

    #[test]
    fn multiplying_by_inverse_of_scaling_matrix() {
        let transform = scaling(2.0, 3.0, 4.0);
        let inv = transform.inverse_gauss_jordan();
        let v = Tuple::vector(-4.0, 6.0, 8.0);
        assert!(&inv * &v == Tuple::vector(-2.0, 2.0, 2.0));
    }

    #[test]
    fn reflection_is_scaling_by_negative_value() {
        let transform = scaling(-1.0, 1.0, 1.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(-2.0, 3.0, 4.0));
    }

    #[test]
    fn rotating_point_around_x_axis() {
        let p = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = rotation_x(std::f64::consts::FRAC_PI_4);
        let full_quarter = rotation_x(std::f64::consts::FRAC_PI_2);
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!((&half_quarter * &p).is_equal(&Tuple::point(0.0, sqrt2_over_2, sqrt2_over_2)));
        assert!((&full_quarter * &p).is_equal(&Tuple::point(0.0, 0.0, 1.0)));
    }

    #[test]
    fn inverse_of_x_rotation_rotates_opposite_direction() {
        let p = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = rotation_x(std::f64::consts::FRAC_PI_4);
        let inv = half_quarter.inverse_gauss_jordan();
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!((&inv * &p).is_equal(&Tuple::point(0.0, sqrt2_over_2, -sqrt2_over_2)));
    }

    #[test]
    fn rotating_point_around_y_axis() {
        let p = Tuple::point(0.0, 0.0, 1.0);
        let half_quarter = rotation_y(std::f64::consts::FRAC_PI_4);
        let full_quarter = rotation_y(std::f64::consts::FRAC_PI_2);
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!((&half_quarter * &p).is_equal(&Tuple::point(sqrt2_over_2, 0.0, sqrt2_over_2)));
        assert!((&full_quarter * &p).is_equal(&Tuple::point(1.0, 0.0, 0.0)));
    }

    #[test]
    fn rotating_point_around_z_axis() {
        let p = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = rotation_z(std::f64::consts::FRAC_PI_4);
        let full_quarter = rotation_z(std::f64::consts::FRAC_PI_2);
        let sqrt2_over_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!((&half_quarter * &p).is_equal(&Tuple::point(-sqrt2_over_2, sqrt2_over_2, 0.0)));
        assert!((&full_quarter * &p).is_equal(&Tuple::point(-1.0, 0.0, 0.0)));
    }

    #[test]
    fn shearing_moves_x_in_proportion_to_y() {
        let transform = shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(5.0, 3.0, 4.0));
    }
    
    #[test]
    fn shearing_moves_x_in_proportion_to_z() {
        let transform = shearing(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(6.0, 3.0, 4.0));
    }
    
    #[test]
    fn shearing_moves_y_in_proportion_to_x() {
        let transform = shearing(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(2.0, 5.0, 4.0));
    }
    
    #[test]
    fn shearing_moves_y_in_proportion_to_z() {
        let transform = shearing(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(2.0, 7.0, 4.0));
    }
    
    #[test]
    fn shearing_moves_z_in_proportion_to_x() {
        let transform = shearing(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(2.0, 3.0, 6.0));
    }
    
    #[test]
    fn shearing_moves_z_in_proportion_to_y() {
        let transform = shearing(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        assert!(&transform * &p == Tuple::point(2.0, 3.0, 7.0));
    }

    #[test]
    fn individual_transformations_applied_in_sequence() {
        let p = Tuple::point(1.0, 0.0, 1.0);
        let a = rotation_x(std::f64::consts::FRAC_PI_2);
        let b = scaling(5.0, 5.0, 5.0);
        let c = translation(10.0, 5.0, 7.0);
    
        let p2 = &a * &p;
        assert!(p2.is_equal(&Tuple::point(1.0, -1.0, 0.0)));
    
        let p3 = &b * &p2;
        assert!(p3.is_equal(&Tuple::point(5.0, -5.0, 0.0)));
    
        let p4 = &c * &p3;
        assert!(p4.is_equal(&Tuple::point(15.0, 0.0, 7.0)));
    }
    
    #[test]
    fn chained_transformations_applied_in_reverse_order() {
        let p = Tuple::point(1.0, 0.0, 1.0);
        let a = rotation_x(std::f64::consts::FRAC_PI_2);
        let b = scaling(5.0, 5.0, 5.0);
        let c = translation(10.0, 5.0, 7.0);
    
        let t = &(&c * &b) * &a;
        assert!((&t * &p).is_equal(&Tuple::point(15.0, 0.0, 7.0)));
    }
}