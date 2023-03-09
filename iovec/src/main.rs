use ucx2_sys::ucp_dt_iov;

#[derive(Debug)]
pub enum Chunk {
    Pointer(*const u8, usize),
    Data(Vec<u8>),
}

pub trait ChunkSerDe {
    fn serialize(&self, chunks: &mut Vec<Chunk>);
    fn deserialize(data: &[u8]) -> Self;
}

#[derive(Debug)]
struct A {
    x: u32,
    y: [f64; 4],
    data: Vec<f64>,
}

impl ChunkSerDe for A {
    fn serialize(&self, chunks: &mut Vec<Chunk>) {
        chunks.push(Chunk::Pointer(std::ptr::addr_of!(self.x) as *const _, std::mem::size_of::<u32>()));
        chunks.push(Chunk::Pointer(self.y.as_ptr() as *const _, self.y.len() * std::mem::size_of::<f64>()));
        // First the length
        let len = self.data.len().to_be_bytes().to_vec();
        chunks.push(Chunk::Data(len));
        // Now the data
        chunks.push(Chunk::Pointer(self.data.as_ptr() as *const _, self.data.len() * std::mem::size_of::<f64>()));
    }

    fn deserialize(data: &[u8]) -> Self {
        unsafe {
            let ptr = data.as_ptr() as *const u32;
            let x = ptr.read_unaligned();
            let ptr = ptr.offset(1) as *const [f64; 4];
            let y: [f64; 4] = ptr.read_unaligned().clone();
            let ptr = ptr.offset(1) as *const u8;
            let len_slice = std::slice::from_raw_parts(ptr, std::mem::size_of::<usize>());
            let len = usize::from_be_bytes(len_slice.try_into().unwrap());
            let mut ptr = ptr.offset(std::mem::size_of::<usize>().try_into().unwrap()) as *const f64;
            let mut data = Vec::new();
            data.reserve(len);
            for _ in 0..len {
                data.push(ptr.read_unaligned());
                ptr = ptr.offset(1);
            }
            Self {
                x,
                y,
                data,
            }
        }
    }
}

fn main() {
    let a = A {
        x: 8,
        y: [1.0, 6.7, 8.9, 11.0],
        data: vec![133.4, 44.90, 7.8],
    };

    let mut chunks = vec![];
    a.serialize(&mut chunks);
    println!("chunks = {:?}", chunks);
    let iovecs: Vec<ucp_dt_iov> = chunks
        .iter()
        .map(|chunk| match chunk {
            Chunk::Pointer(buffer, length) => ucp_dt_iov {
                buffer: *buffer as *mut _,
                length: *length,
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
            Chunk::Pointer(buffer, length) => unsafe {
                std::slice::from_raw_parts(*buffer, *length)
            }
            Chunk::Data(data) => &data[..],
        })
        .flatten()
        .map(|i| *i)
        .collect();
    let new_a = A::deserialize(&data);
    println!("new_a = {:?}", new_a);
}
