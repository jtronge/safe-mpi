use ucx2_sys::ucp_dt_iov;

#[derive(Debug)]
pub enum Chunk {
    Pointer(*const u8, usize),
    Data(Vec<u8>),
}

pub trait ChunkSerDe {
    fn serialize(&self, chunks: &mut Vec<Chunk>);
    fn deserialize(&self, data: &[u8]) -> Self;
}

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

    fn deserialize(&self, data: &[u8]) -> Self {
        Self {
            x: 100,
            y: [1.0, 2.0, 3.0, 4.0],
            data: vec![],
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
}
