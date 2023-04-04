use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    TypeMismatch,
    MissingTypeID,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ChunkSerDe: Sized {
    fn serialize<'a>(data: &'a [Self], chunks: &mut Vec<Chunk<'a>>) -> Result<()>;
    fn deserialize(data: &[u8]) -> Result<(Vec<Self>, &[u8])>;
}

#[inline]
pub fn hash_type_id<T: ?Sized + 'static>() -> u64 {
    let mut hasher = DefaultHasher::new();
    let type_id = TypeId::of::<T>();
    type_id.hash(&mut hasher);
    hasher.finish()
}

/// Add the type ID of some type to the header.
#[inline]
pub fn add_type_id_header<T: ?Sized + 'static>(chunks: &mut Vec<Chunk>) {
    let type_id = hash_type_id::<T>().to_be_bytes();
    chunks.push(Chunk::Data(type_id.to_vec()));
}

/// Add a length as a chunk to the header.
#[inline]
pub fn add_length_header(chunks: &mut Vec<Chunk>, len: usize) {
    let len = len.to_be_bytes();
    chunks.push(Chunk::Data(len.to_vec()));
}

/// Check the type ID contained at the start of the data passed in. Return the
/// rest of the data if OK.
#[inline]
pub fn check_type_id_header<T: ?Sized + 'static>(data: &[u8]) -> Result<&[u8]> {
    if data.len() < std::mem::size_of::<u64>() {
        return Err(Error::MissingTypeID);
    }
    let type_id = u64::from_be_bytes(data[..std::mem::size_of::<u64>()].try_into().unwrap());
    if type_id != hash_type_id::<T>() {
        return Err(Error::TypeMismatch);
    }
    Ok(&data[std::mem::size_of::<u64>()..])
}

/// Check the length stored at the start of the data passed in. Return the rest
/// if it's OK.
#[inline]
pub fn check_length_header<T>(data: &[u8]) -> Result<(usize, &[u8])> {
    if data.len() < std::mem::size_of::<usize>() {
        return Err(Error::MissingLength);
    }
    let len = usize::from_be_bytes(data[..std::mem::size_of::<usize>()].try_into().unwrap());
    let data = &data[std::mem::size_of::<usize>()..];
    if data.len() < (std::mem::size_of::<T>() * len) {
        println!("got: {}", data.len());
        println!("exp: {}", std::mem::size_of::<T>() * len);
        return Err(Error::MissingData);
    }
    Ok((len, data))
}

macro_rules! impl_chunkserde {
    ($ty:ident) => {
        impl ChunkSerDe for $ty {
            fn serialize(data: &[Self], chunks: &mut Vec<Chunk>) -> Result<()> {
                unsafe {
                    add_type_id_header::<$ty>(chunks);
                    add_length_header(chunks, data.len());
                    let ptr = data.as_ptr() as *const u8;
                    let slice =
                        std::slice::from_raw_parts(ptr, data.len() * std::mem::size_of::<$ty>());
                    chunks.push(Chunk::Slice(slice));
                    Ok(())
                }
            }

            fn deserialize(data: &[u8]) -> Result<(Vec<Self>, &[u8])> {
                unsafe {
                    let data = check_type_id_header::<$ty>(data)?;
                    let (len, data) = check_length_header::<$ty>(data)?;
                    let mut out = vec![];
                    let mut ptr = data.as_ptr() as *const $ty;
                    for _ in 0..len {
                        out.push(ptr.read_unaligned());
                        ptr = ptr.offset(1);
                    }
                    let used = std::mem::size_of::<$ty>() * len;
                    Ok((out, &data[used..]))
                }
            }
        }
    };
}

impl_chunkserde!(i32);

/*
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
*/
