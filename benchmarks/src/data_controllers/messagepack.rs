use serde::{Serialize, de::DeserializeOwned};
use rmp_serde;
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

pub struct MessagePackController {
    pub comm: Communicator,
}

impl MessagePackController {
    pub fn new(comm: Communicator) -> MessagePackController {
        MessagePackController {
            comm,
        }
    }
}

impl SerdeController for MessagePackController {
    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = rmp_serde::to_vec(data)
            .map_err(|_| Error::SerializeError)?;
        let buf = Data::Contiguous(&buf);
        self.comm.send(buf, tag)
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv(tag)?;
        rmp_serde::decode::from_slice(&buf)
            .map_err(|_| Error::DeserializeError)
    }
}
