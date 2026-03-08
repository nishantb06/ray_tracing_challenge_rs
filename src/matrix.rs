use crate::utils::{equal};
use crate::tuple::Tuple;
use std::ops::Mul;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Matrix {
    pub rows: usize,
    pub columns: usize,
    pub data: Vec<Vec<f64>>
}

#[allow(dead_code)]
impl Matrix {
    pub fn new(rows: usize, columns: usize) -> Self {
        Matrix{
            rows,
            columns,
            data: vec![vec![0.0; columns as usize]; rows as usize],
        }
    }

    pub fn new_with_data(rows: usize, columns: usize, data: Vec<f64>) -> Self{
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

    pub fn identity(size: usize) -> Self {
        let mut m = Matrix::new(size, size);
        for i in 0..size {
            m.data[i][i] = 1.0;
        }
        m
    }

    pub fn transpose(&self) -> Self {
        let mut result = Matrix::new(self.columns, self.rows);
        for i in 0..self.rows {
            for j in 0..self.columns {
                result.data[j][i] = self.data[i][j];
            }
        }
        result
    }

    pub fn transpose_owned(self) -> Self {
        let mut result = Matrix::new(self.columns, self.rows);
        for i in 0..self.rows {
            for j in 0..self.columns {
                result.data[j][i] = self.data[i][j];
            }
        }
        result
    }

    pub fn transpose_in_place(&mut self) {
        assert_eq!(self.rows, self.columns, "In-place transpose only works for square matrices");
        for i in 0..self.rows {
            for j in (i + 1)..self.columns {
                let tmp = self.data[i][j];
                self.data[i][j] = self.data[j][i];
                self.data[j][i] = tmp;
            }
        }
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        if self.rows != other.rows || self.columns != other.columns {
            return false;
        }
        for i in 0..self.rows as usize {
            for j in 0..self.columns as usize {
                if !equal(self.data[i][j], other.data[i][j]) {
                    return false;
                }
            }
        }
        true
    }
}

impl Mul for &Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        // validate dimensions: self.columns must equal rhs.rows
        if self.columns != rhs.rows {
            panic!(
                "Cannot multiply matrices: left is {}x{}, right is {}x{}",
                self.rows, self.columns, rhs.rows, rhs.columns
            )
        }
        // pick one row from the lhs and take the weighted sum of the rows of the rhs 
        // considering the values from the choses row in lhs as weights
        // the resulting row will be inserted into the final matrix.


        let mut final_matrix: Vec<Vec<f64>> = Vec::new();

        for r in 0..self.rows {
            let mut new_row = vec![0.0; rhs.columns as usize];

            for i in 0..self.columns {
                let multiplier = self.data[r][i];
                let row = &rhs.data[i];

                for j in 0..rhs.columns {
                    let val = row[j];
                    new_row[j] += multiplier * val;
                }
            }

            final_matrix.push(new_row);
        }

        return Matrix::new_with_data(
            self.rows,
            rhs.columns,
            final_matrix.into_iter().flatten().collect()
        );
    }
}

impl Mul<&Tuple> for &Matrix {
    type Output = Tuple;

    fn mul(self, rhs: &Tuple) -> Self::Output {
        assert!(self.columns == 4, "Matrix must have 4 columns to multiply with a Tuple");
        let tuple_col = [rhs.x, rhs.y, rhs.z, rhs.w];
        let mut result = [0.0; 4];
        for r in 0..self.rows {
            for c in 0..4 {
                result[r] += self.data[r][c] * tuple_col[c];
            }
        }
        Tuple::new(result[0], result[1], result[2], result[3])
    }
}

impl Mul<&Matrix> for &Tuple {
    type Output = Tuple;

    fn mul(self, rhs: &Matrix) -> Self::Output {
        assert!(rhs.rows == 4, "Matrix must have 4 rows to multiply with a Tuple");
        let tuple_row = [self.x, self.y, self.z, self.w];
        let mut result = [0.0; 4];
        for c in 0..rhs.columns {
            for r in 0..4 {
                result[c] += tuple_row[r] * rhs.data[r][c];
            }
        }
        Tuple::new(result[0], result[1], result[2], result[3])
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

    #[test]
    fn matrix_equality_with_identical_matrices() {
        let data = vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 8.0, 7.0, 6.0,
            5.0, 4.0, 3.0, 2.0,
        ];
        let m1 = Matrix::new_with_data(4, 4, data.clone());
        let m2 = Matrix::new_with_data(4, 4, data);
        assert!(&m1 == &m2);
        assert!(m1 == m2);
    }

    #[test]
    fn matrix_equality_with_different_matrices() {
        let a = Matrix::new_with_data(4, 4, vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 8.0, 7.0, 6.0,
            5.0, 4.0, 3.0, 2.0,
        ]);
        let b = Matrix::new_with_data(4, 4, vec![
            2.0, 3.0, 4.0, 5.0,
            6.0, 7.0, 8.0, 9.0,
            8.0, 7.0, 6.0, 5.0,
            4.0, 3.0, 2.0, 1.0,
        ]);
        assert!(a != b);
    }

    #[test]
    fn multiplying_two_matrices() {
        let a = Matrix::new_with_data(4, 4, vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 8.0, 7.0, 6.0,
            5.0, 4.0, 3.0, 2.0,
        ]);
        let b = Matrix::new_with_data(4, 4, vec![
            -2.0, 1.0, 2.0, 3.0,
             3.0, 2.0, 1.0,-1.0,
             4.0, 3.0, 6.0, 5.0,
             1.0, 2.0, 7.0, 8.0,
        ]);
        let expected = Matrix::new_with_data(4, 4, vec![
            20.0, 22.0, 50.0, 48.0,
            44.0, 54.0,114.0,108.0,
            40.0, 58.0,110.0,102.0,
            16.0, 26.0, 46.0, 42.0,
        ]);
        assert!(&a * &b == expected);
    }

    use crate::tuple::Tuple;

    #[test]
    fn matrix_multiplied_by_tuple() {
        let a = Matrix::new_with_data(4, 4, vec![
            1.0, 2.0, 3.0, 4.0,
            2.0, 4.0, 4.0, 2.0,
            8.0, 6.0, 4.0, 1.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        let b = Tuple::new(1.0, 2.0, 3.0, 1.0);
        let expected = Tuple::new(18.0, 24.0, 33.0, 1.0);
        assert!((&a * &b).is_equal(&expected));
    }

    #[test]
    fn tuple_multiplied_by_matrix() {
        let a = Tuple::new(1.0, 2.0, 3.0, 1.0);
        let b = Matrix::new_with_data(4, 4, vec![
            1.0, 2.0, 3.0, 4.0,
            2.0, 4.0, 4.0, 2.0,
            8.0, 6.0, 4.0, 1.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        let expected = Tuple::new(29.0, 28.0, 23.0, 12.0);
        assert!((&a * &b).is_equal(&expected));
    }

    #[test]
    fn multiplying_matrix_by_identity_matrix() {
        let a = Matrix::new_with_data(4, 4, vec![
            0.0, 1.0,  2.0,  4.0,
            1.0, 2.0,  4.0,  8.0,
            2.0, 4.0,  8.0, 16.0,
            4.0, 8.0, 16.0, 32.0,
        ]);
        let identity = Matrix::identity(4);
        assert!(&a * &identity == a);
    }
    
    #[test]
    fn multiplying_identity_matrix_by_tuple() {
        let identity = Matrix::identity(4);
        let a = Tuple::new(1.0, 2.0, 3.0, 4.0);
        let expected = Tuple::new(1.0, 2.0, 3.0, 4.0);
        assert!((&identity * &a).is_equal(&expected));
    }

    #[test]
    fn transposing_a_matrix() {
        let a = Matrix::new_with_data(4, 4, vec![
            0.0, 9.0, 3.0, 0.0,
            9.0, 8.0, 0.0, 8.0,
            1.0, 8.0, 5.0, 3.0,
            0.0, 0.0, 5.0, 8.0,
        ]);
        let expected = Matrix::new_with_data(4, 4, vec![
            0.0, 9.0, 1.0, 0.0,
            9.0, 8.0, 8.0, 0.0,
            3.0, 0.0, 5.0, 5.0,
            0.0, 8.0, 3.0, 8.0,
        ]);
        let a_t = a.transpose();
        assert!(a_t == expected);
        // a is still valid after transpose
        assert!(a == Matrix::new_with_data(4, 4, vec![
            0.0, 9.0, 3.0, 0.0,
            9.0, 8.0, 0.0, 8.0,
            1.0, 8.0, 5.0, 3.0,
            0.0, 0.0, 5.0, 8.0,
        ]));
    }
}