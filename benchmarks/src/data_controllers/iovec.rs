use iovec::{ChunkSerDe, Chunk};
use safe_mpi::{
    Result,
    Tag,
    Error,
    communicator::{
        Communicator,
        Data,
    },
};

pub struct IovecController {
    pub comm: Communicator,
}

impl IovecController {
    pub fn new(comm: Communicator) -> IovecController {
        IovecController {
            comm,
        }
    }

    pub fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: ChunkSerDe,
    {
        let mut chunks = vec![];
        data.serialize(&mut chunks)
            .map_err(|_| Error::SerializeError)?;
        let send_data: Vec<&[u8]> = chunks
            .iter()
            .map(|chunk| match chunk {
                Chunk::Slice(slice) => slice,
                Chunk::Data(data) => &data[..],
            })
            .collect();
        self.comm.send(Data::Chunked(&send_data[..]), tag)
    }

    pub fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: ChunkSerDe,
    {
        let buf = self.comm.recv(tag)?;
        let (data, size) = T::deserialize(&buf)
            .map_err(|_| Error::DeserializeError)?;
        Ok(data)
    }
}
