// use crate::utils::{equal};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Matrix {
    pub rows: i32,
    pub columns: i32,
    pub data: Vec<Vec<f64>>
}

#[allow(dead_code)]
impl Matrix {
    pub fn new(rows: i32, columns: i32) -> Self {
        Matrix{
            rows,
            columns,
            data: vec![vec![0.0; columns as usize]; rows as usize],
        }
    }

    pub fn new_with_data(rows: i32, columns: i32, data: Vec<f64>) -> Self{
        assert_eq!(
            data.len(),
            (rows * columns) as usize,
            "Data length does not match matrix dimensions"
        );
        let mut data_iter = data.into_iter();
        let matrix: Vec<Vec<f64>> = (0..rows)
            .map(|_| {
                (0..columns)
                    .map(|_| data_iter.next().unwrap())
                    .collect()
            })
            .collect();

        Matrix {
            rows,
            columns,
            data: matrix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_and_inspect_4x4_matrix() {
        let data = vec![
            1.0, 2.0, 3.0, 4.0,
            5.5, 6.5, 7.5, 8.5,
            9.0, 10.0, 11.0, 12.0,
            13.5, 14.5, 15.5, 16.5,
        ];
        let m = Matrix::new_with_data(4,4,data);
        assert_eq!(m.data[0][0], 1.0);    // M[0,0]
        assert_eq!(m.data[0][3], 4.0);    // M[0,3]
        assert_eq!(m.data[1][0], 5.5);    // M[1,0]
        assert_eq!(m.data[1][2], 7.5);    // M[1,2]
        assert_eq!(m.data[2][2], 11.0);   // M[2,2]
        assert_eq!(m.data[3][0], 13.5);   // M[3,0]
        assert_eq!(m.data[3][2], 15.5);   // M[3,2]
    }

    #[test]
    fn construct_and_inspect_2x2_matrix() {
        // | -3 |  5 |
        // |  1 | -2 |
        let data = vec![
            -3.0, 5.0,
             1.0, -2.0
        ];
        let m = Matrix::new_with_data(2, 2, data);
        assert_eq!(m.data[0][0], -3.0);  // M[0,0]
        assert_eq!(m.data[0][1], 5.0);   // M[0,1]
        assert_eq!(m.data[1][0], 1.0);   // M[1,0]
        assert_eq!(m.data[1][1], -2.0);  // M[1,1]
    }

    #[test]
    fn construct_and_inspect_3x3_matrix() {
        // | -3 |  5 |  0 |
        // |  1 | -2 | -7 |
        // |  0 |  1 |  1 |
        let data = vec![
            -3.0, 5.0, 0.0,
             1.0, -2.0, -7.0,
             0.0, 1.0, 1.0
        ];
        let m = Matrix::new_with_data(3, 3, data);
        assert_eq!(m.data[0][0], -3.0);  // M[0,0]
        assert_eq!(m.data[1][1], -2.0);  // M[1,1]
        assert_eq!(m.data[2][2], 1.0);   // M[2,2]
    }

}