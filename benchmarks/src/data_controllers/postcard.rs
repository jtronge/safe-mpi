use serde::{Serialize, de::DeserializeOwned};
use postcard;
use safe_mpi::{
    Result,
    Error,
    Tag,
    communicator::{
        Communicator,
        Data,
    },
};
use crate::data_controllers::SerdeController;

pub struct PostcardController {
    pub comm: Communicator,
}

impl PostcardController {
    pub fn new(comm: Communicator) -> PostcardController {
        PostcardController {
            comm,
        }
    }
}

impl SerdeController for PostcardController {
    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = postcard::to_allocvec(data)
            .map_err(|_| Error::SerializeError)?;
        let buf = Data::Contiguous(&buf);
        self.comm.send(buf, tag)
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv(tag)?;
        postcard::from_bytes(&buf)
            .map_err(|_| Error::DeserializeError)
    }
}
