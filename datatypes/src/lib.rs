use flat_derive::FlatBuffer;
use iovec::{
    add_length_header, add_type_id_header, check_length_header, check_type_id_header, Chunk,
    ChunkSerDe, Result,
};
use mpi::traits::Equivalence;
use serde::{Deserialize, Serialize};

mod datatype;
pub use datatype::DataType;

/// Generate a vector of a simple type of the given size (roughly).
pub fn simple(size: usize) -> Vec<i32> {
    let count = size / std::mem::size_of::<i32>();
    (0..count.try_into().unwrap()).collect()
}

const X_ITEM_COUNT: usize = 16;

#[derive(Serialize, Deserialize, Equivalence, FlatBuffer, Default)]
pub struct ComplexNoncompound {
    i: i32,
    d: f64,
    x: [f32; X_ITEM_COUNT],
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
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                ],
            }
        })
        .collect()
}

impl ChunkSerDe for ComplexNoncompound {
    fn serialize(data: &[Self], chunks: &mut Vec<Chunk>) -> Result<()> {
        unsafe {
            add_type_id_header::<Self>(chunks);
            add_length_header(chunks, data.len());
            let ptr = data.as_ptr() as *const u8;
            let slice = std::slice::from_raw_parts(
                ptr,
                data.len() * std::mem::size_of::<ComplexNoncompound>(),
            );
            chunks.push(Chunk::Slice(slice));
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Vec<Self>, &[u8])> {
        unsafe {
            let data = check_type_id_header::<Self>(data)?;
            let (len, data) = check_length_header::<Self>(data)?;
            let mut ptr = data.as_ptr() as *const ComplexNoncompound;
            let mut out = vec![];
            for _ in 0..len {
                out.push(ptr.read_unaligned());
                ptr = ptr.offset(1);
            }
            let off = std::mem::size_of::<ComplexNoncompound>() * len;
            Ok((out, &data[off..]))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComplexCompound {
    i: i32,
    d: f64,
    x: Vec<f32>,
}

pub fn complex_compound(size: usize) -> Vec<ComplexCompound> {
    let elm_size = std::mem::size_of::<i32>()
        + std::mem::size_of::<f64>()
        + std::mem::size_of::<usize>()
        + std::mem::size_of::<f32>() * X_ITEM_COUNT;
    let count = size / elm_size;
    (0..count.try_into().unwrap())
        .map(|i| {
            let d = i as f64;
            let f = i as f32;
            ComplexCompound {
                i,
                d,
                x: vec![
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                    0.01 * f,
                    0.06 * f,
                    f,
                    0.1 * f,
                ],
            }
        })
        .collect()
}

impl ChunkSerDe for ComplexCompound {
    fn serialize(data: &[Self], chunks: &mut Vec<Chunk>) -> Result<()> {
        unsafe {
            add_type_id_header::<Self>(chunks);
            add_length_header(chunks, data.len());
            for elm in data {
                chunks.push(Chunk::Slice(std::slice::from_raw_parts(
                    (&elm.i as *const i32) as *const u8,
                    std::mem::size_of::<i32>(),
                )));
                chunks.push(Chunk::Slice(std::slice::from_raw_parts(
                    (&elm.d as *const f64) as *const u8,
                    std::mem::size_of::<f64>(),
                )));
                let len = elm.x.len().to_be_bytes().to_vec();
                chunks.push(Chunk::Data(len));
                let slice = std::slice::from_raw_parts(
                    elm.x.as_ptr() as *const _,
                    elm.x.len() * std::mem::size_of::<i32>(),
                );
                chunks.push(Chunk::Slice(slice));
            }
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Vec<Self>, &[u8])> {
        unsafe {
            let data = check_type_id_header::<Self>(data)?;
            let (count, mut data) = check_length_header::<Self>(data)?;
            let mut out = vec![];
            for _ in 0..count {
                let ptr = data.as_ptr() as *const i32;
                let i = ptr.read_unaligned();
                let ptr = ptr.offset(1) as *const f64;
                let d = ptr.read_unaligned();
                let ptr = ptr.offset(1) as *const u8;
                let len_bytes = std::slice::from_raw_parts(ptr, std::mem::size_of::<usize>());
                let len = usize::from_be_bytes(len_bytes.try_into().unwrap());
                assert_eq!(len, X_ITEM_COUNT);
                let ptr = ptr.offset(std::mem::size_of::<usize>().try_into().unwrap());
                let mut x = vec![0.0; X_ITEM_COUNT];
                std::ptr::copy(
                    ptr,
                    x.as_mut_ptr() as *mut u8,
                    X_ITEM_COUNT * std::mem::size_of::<f32>(),
                );
                let size = std::mem::size_of::<i32>()
                    + std::mem::size_of::<f64>()
                    + std::mem::size_of::<usize>()
                    + std::mem::size_of::<f32>() * len;
                data = &data[size..];
                out.push(ComplexCompound { i, d, x });
            }
            Ok((out, data))
        }
    }
}
