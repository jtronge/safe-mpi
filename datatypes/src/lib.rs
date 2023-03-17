use serde::{
    Serialize,
    Deserialize,
};
use mpi::traits::Equivalence;
use iovec::{
    ChunkSerDe,
    Chunk,
    Result,
};
use memoffset::offset_of;

mod datatype;
pub use datatype::DataType;

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

impl ChunkSerDe for ComplexNoncompound {
    fn serialize(&self, chunks: &mut Vec<Chunk>) -> Result<()> {
        unsafe {
            let ptr = std::ptr::addr_of!(self) as *const u8;
            let slice = std::slice::from_raw_parts(
                ptr,
                std::mem::size_of::<ComplexNoncompound>(),
            );
            chunks.push(Chunk::Slice(slice));
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize)> {
        unsafe {
            assert!(data.len() >= std::mem::size_of::<ComplexNoncompound>());
            let ptr = data.as_ptr() as *const ComplexNoncompound;
            Ok((
                ptr.read_unaligned(),
                std::mem::size_of::<ComplexNoncompound>(),
            ))
        }
    }
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

impl ChunkSerDe for ComplexCompound {
    fn serialize(&self, chunks: &mut Vec<Chunk>) -> Result<()> {
        unsafe {
            let ptr = std::ptr::addr_of!(self) as *const u8;
            let slice = std::slice::from_raw_parts(
                ptr,
                std::mem::size_of::<ComplexCompound>(),
            );
            chunks.push(Chunk::Slice(slice));
            let len = self.data.len().to_be_bytes().to_vec();
            chunks.push(Chunk::Data(len));
            let slice = std::slice::from_raw_parts(
                self.data.as_ptr() as *const _,
                self.data.len() * std::mem::size_of::<i32>(),
            );
            chunks.push(Chunk::Slice(slice));
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize)> {
        unsafe {
            let ptr = data.as_ptr();
            let off = offset_of!(ComplexCompound, m);
            let m = (ptr.offset(off.try_into().unwrap()) as *const i32).read_unaligned();
            let off = offset_of!(ComplexCompound, n);
            let n = (ptr.offset(off.try_into().unwrap()) as *const i32).read_unaligned();
            let ptr = ptr.offset(std::mem::size_of::<ComplexCompound>().try_into().unwrap());
            let len_bytes = std::slice::from_raw_parts(
                ptr,
                std::mem::size_of::<usize>(),
            );
            let len = usize::from_be_bytes(len_bytes.try_into().unwrap());
            let mut ptr = ptr.offset(std::mem::size_of::<usize>().try_into().unwrap()) as *const i32;
            let mut data = Vec::new();
            for _ in 0..len {
                data.push(ptr.read_unaligned());
                ptr = ptr.offset(1);
            }
            let size = std::mem::size_of::<ComplexCompound>()
                       + std::mem::size_of::<usize>()
                       + std::mem::size_of::<i32>() * len;
            Ok((
                ComplexCompound {
                    m,
                    n,
                    data,
                },
                size,
            ))
        }
    }
}
