use std::slice::from_raw_parts;
use ucx2_sys::ucp_dt_iov;
use iovec::{Chunk, ChunkSerDe, Result, add_length_header, check_length_header};

#[derive(Debug)]
struct A {
    x: u32,
    y: [f64; 4],
    data: Vec<f64>,
}

impl ChunkSerDe for A {
    fn serialize(data: &[Self], chunks: &mut Vec<Chunk>) -> Result<()> {
        // NOTE: No type ID here
        unsafe {
            add_length_header(chunks, data.len());
            for elm in data {
                chunks.push(Chunk::Slice(
                    from_raw_parts(
                        std::ptr::addr_of!(elm.x) as *const _,
                        std::mem::size_of::<u32>(),
                    )
                ));
                chunks.push(Chunk::Slice(
                    from_raw_parts(
                        elm.y.as_ptr() as *const _,
                        elm.y.len() * std::mem::size_of::<f64>(),
                    )
                ));
                // First the length
                let len = elm.data.len().to_be_bytes().to_vec();
                chunks.push(Chunk::Data(len));
                // Now the data
                chunks.push(Chunk::Slice(
                    from_raw_parts(
                        elm.data.as_ptr() as *const _,
                        elm.data.len() * std::mem::size_of::<f64>(),
                    )
                ));
            }
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Vec<Self>, &[u8])> {
        let (count, new_data) = check_length_header::<Self>(data)?;
        let mut data = new_data;
        let mut out = vec![];
        unsafe {
            for _ in 0..count {
                let ptr = data.as_ptr() as *const u32;
                let x = ptr.read_unaligned();
                let ptr = ptr.offset(1) as *const [f64; 4];
                let y: [f64; 4] = ptr.read_unaligned().clone();
                let ptr = ptr.offset(1) as *const u8;
                let len_slice = std::slice::from_raw_parts(ptr, std::mem::size_of::<usize>());
                let len = usize::from_be_bytes(len_slice.try_into().unwrap());
                let mut ptr = ptr.offset(std::mem::size_of::<usize>().try_into().unwrap()) as *const f64;
                let mut sdata = Vec::new();
                sdata.reserve(len);
                for _ in 0..len {
                    sdata.push(ptr.read_unaligned());
                    ptr = ptr.offset(1);
                }
                let used = std::mem::size_of::<u32>()
                           + 4 * std::mem::size_of::<f64>()
                           + std::mem::size_of::<usize>()
                           + len * std::mem::size_of::<f64>();
                data = &data[used..];
                out.push(A {
                    x,
                    y,
                    data: sdata,
                });
            }
        }
        Ok((out, &data[..]))
    }
}

fn main() {
    let a = [
        A {
            x: 8,
            y: [1.0, 6.7, 8.9, 11.0],
            data: vec![133.4, 44.90, 7.8],
        },
    ];

    let mut chunks = vec![];
    A::serialize(&a, &mut chunks).unwrap();
    println!("chunks = {:?}", chunks);
    let iovecs: Vec<ucp_dt_iov> = chunks
        .iter()
        .map(|chunk| match chunk {
            Chunk::Slice(buf) => ucp_dt_iov {
                buffer: buf.as_ptr() as *mut _,
                length: buf.len(),
            },
            Chunk::Data(data) => ucp_dt_iov {
                buffer: data.as_ptr() as *mut _,
                length: data.len(),
            },
        })
        .collect();
    println!("iovecs = {:?}", iovecs);
    let data: Vec<u8> = chunks
        .iter()
        .map(|chunk| match chunk {
            Chunk::Slice(buf) => buf,
            Chunk::Data(data) => &data[..],
        })
        .flatten()
        .map(|i| *i)
        .collect();
    let (new_a, _) = A::deserialize(&data).unwrap();
    println!("new_a = {:?}", new_a);
}
