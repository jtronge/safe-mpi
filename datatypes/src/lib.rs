use serde::{Serialize, Deserialize};
use mpi::traits::Equivalence;

/// Generate a vector of a simple type of the given size (roughly).
pub fn simple(size: usize) -> Vec<i32> {
    let count = size / std::mem::size_of::<i32>();
    (0..count.try_into().unwrap()).collect()
}

#[derive(Serialize, Deserialize, Equivalence)]
pub struct ComplexNoncompound {
    i: i32,
    d: f64,
    x: [f32; 16],
}

// Tile, bucket, and alternating (from Xiong et al.)
pub fn complex_noncompound(size: usize) -> Vec<ComplexNoncompound> {
    let count = size / std::mem::size_of::<ComplexNoncompound>();
    (0..count.try_into().unwrap())
        .map(|i| {
            let d = i as f64;
            let f = i as f32;
            ComplexNoncompound {
                i,
                d,
                x: [
                    0.01 * f, 0.06 * f, f, 0.1 * f,
                    0.01 * f, 0.06 * f, f, 0.1 * f,
                    0.01 * f, 0.06 * f, f, 0.1 * f,
                    0.01 * f, 0.06 * f, f, 0.1 * f,
                ],
            }
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct ComplexCompound {
    m: i32,
    n: i32,
    data: Vec<i32>
}

pub fn complex_compound(size: usize) -> Vec<ComplexCompound> {
    let mut total_size = 0;
    let mut out = vec![];
    for i in 0..size {
        let len = if (i % 2) == 0 { 128 } else { 256 };
        // Estimate of the amount of memory for this item
        let next_size = std::mem::size_of::<ComplexCompound>() + std::mem::size_of::<i32>() * len;
        // Check if this next size will be too big
        if (total_size + next_size) > size {
            break;
        }
        let i: i32 = i.try_into().unwrap();
        out.push(ComplexCompound {
            m: i * 3,
            n: i * 2,
            data: (0..len.try_into().unwrap()).collect(),
        });
        total_size += next_size;
    }
    out
}
