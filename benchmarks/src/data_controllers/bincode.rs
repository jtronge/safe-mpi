use serde::{Serialize, de::DeserializeOwned};
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

pub struct BincodeController {
    pub comm: Communicator,
}

impl BincodeController {
    pub fn new(comm: Communicator) -> BincodeController {
        BincodeController {
            comm,
        }
    }
}

impl SerdeController for BincodeController {
    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = bincode::serialize(data)
            .map_err(|_| Error::SerializeError)?;
        let buf = Data::Contiguous(&buf);
        self.comm.send(buf, tag)
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv(tag)?;
        bincode::deserialize(&buf)
            .map_err(|_| Error::DeserializeError)
    }
}
