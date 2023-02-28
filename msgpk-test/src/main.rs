use serde::{Serialize, Deserialize};
use rmp_serde::{
    self,
    encode,
};

#[derive(Debug, Serialize, Deserialize)]
struct A {
    x: u32,
    y: [f64; 2],
}

fn main() {
    let a = A { x: 6, y: [1.0, 3.6] };

    let mut data = vec![];
    encode::write(&mut data, &a).unwrap();
    println!("{:?}", data);
    let a2: A = rmp_serde::from_read(&*data).unwrap();
    println!("{:?}", a2);
}
