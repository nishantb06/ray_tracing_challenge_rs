mod tuple;
mod utils;

use tuple::Tuple;

fn main() {
    println!("Hello, world!");
    // let t = Tuple{
    //     x : 1.0,
    //     y : 3.0,
    //     z : 4.0,
    //     w : 0.0,
    // };
    let t = Tuple::vector(1.0, 3.0, 4.0);
    println!("{:?}", t);
}
