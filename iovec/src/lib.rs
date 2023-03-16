#[derive(Debug)]
pub enum Chunk<'a> {
    Slice(&'a [u8]),
    Data(Vec<u8>),
}

pub trait ChunkSerDe {
    fn serialize<'a>(&self, chunks: &mut Vec<Chunk<'a>>);
    fn deserialize(data: &[u8]) -> Self;
}

macro_rules! impl_vec {
    ($ty:ident) => {
        impl ChunkSerDe for Vec<$ty> {
            fn serialize(&self, chunks: &mut Vec<Chunk>) {
                unsafe {
                    let ptr = self.as_ptr() as *const u8;
                    let slice = std::slice::from_raw_parts(
                        ptr,
                        self.len() * std::mem::size_of::<$ty>(),
                    );
                    chunks.push(Chunk::Slice(slice));
                }
            }

            fn deserialize(data: &[u8]) -> Self {
                unsafe {
                    let len = data.len() / std::mem::size_of::<$ty>();
                    let mut out = Vec::new();
                    let mut ptr = data.as_ptr() as *const $ty;
                    for _ in 0..len {
                        out.push(ptr.read_unaligned());
                        ptr = ptr.offset(1);
                    }
                    out
                }
            }
        }
    }
}

impl_vec!(i32);
