#[derive(Debug)]
pub enum Chunk<'a> {
    Slice(&'a [u8]),
    Data(Vec<u8>),
}

#[derive(Clone, Debug)]
pub enum Error {
    SerializeError,
    DeserializeError,
    MissingLength,
    MissingData,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ChunkSerDe {
    fn serialize<'a>(&self, chunks: &mut Vec<Chunk<'a>>) -> Result<()>;
    fn deserialize(data: &[u8]) -> Result<(Self, usize)> where Self: Sized;
}

macro_rules! impl_vec {
    ($ty:ident) => {
        impl ChunkSerDe for Vec<$ty> {
            fn serialize(&self, chunks: &mut Vec<Chunk>) -> Result<()> {
                unsafe {
                    let len = self.len().to_be_bytes();
                    chunks.push(Chunk::Data(len.to_vec()));
                    let ptr = self.as_ptr() as *const u8;
                    let slice = std::slice::from_raw_parts(
                        ptr,
                        self.len() * std::mem::size_of::<$ty>(),
                    );
                    chunks.push(Chunk::Slice(slice));
                    Ok(())
                }
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize)> {
                if data.len() < std::mem::size_of::<usize>() {
                    return Err(Error::MissingLength);
                }
                let len = usize::from_be_bytes(data[..std::mem::size_of::<usize>()].try_into().unwrap());
                let data = &data[std::mem::size_of::<usize>()..];
                if data.len() < (std::mem::size_of::<$ty>() * len) {
                    return Err(Error::MissingData);
                }
                unsafe {
                    // TODO: This needs a count
                    let mut out = Vec::new();
                    let mut ptr = data.as_ptr() as *const $ty;
                    for _ in 0..len {
                        out.push(ptr.read_unaligned());
                        ptr = ptr.offset(1);
                    }
                    Ok((
                        out,
                        std::mem::size_of::<usize>() + std::mem::size_of::<$ty>() * len,
                    ))
                }
            }
        }
    }
}

impl_vec!(i32);

/*
impl<T> ChunkSerDe for Vec<T>
where
    T: ChunkSerDe,
{
    fn serialize(&self, chunks: &mut Vec<Chunk>) {
    }

    fn deserialize(data: &[u8]) -> Self {
    }
}
*/

impl<T> ChunkSerDe for Vec<T>
where
    T: ChunkSerDe,
{
    fn serialize(&self, chunks: &mut Vec<Chunk>) -> Result<()> {
        // Push the count
        let count = self.len().to_be_bytes().to_vec();
        chunks.push(Chunk::Data(count));
        unsafe {
            for val in self {
                val.serialize(chunks)?;
            }
            Ok(())
        }
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize)> {
        // Get the count
        let mut i = 0;
        let count = usize::from_be_bytes(data[..std::mem::size_of::<usize>()].try_into().unwrap());
        i += std::mem::size_of::<usize>();
        unsafe {
            let mut out = vec![];
            for j in 0..count {
                let (val, size) = T::deserialize(&data[i..])?;
                i += size;
                out.push(val);
            }
            Ok((out, i))
        }
    }
}
