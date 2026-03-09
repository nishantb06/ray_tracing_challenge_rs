mod tuple;
mod utils;
mod matrix;
mod transformation;

use matrix::Matrix;
fn main() {
    let m: Matrix = Matrix::new_with_data(2,2, vec![1.0,2.0,3.0,4.0]);
    println!("{:?}", m);
}