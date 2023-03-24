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
use crate::data_controllers::serde::{SerdeController, SerdeScope, SerdeRequestStatus};

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
    type Scope = MessagePackScope;

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

    fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Scope) -> R,
    {
        f(&MessagePackScope)
    }
}

pub struct MessagePackScope;

impl SerdeScope for MessagePackScope {
    type Request = usize;

    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<Self::Request>
    where
        T: Serialize + DeserializeOwned
    {
        Ok(0)
    }

    fn irecv(&mut self, tag: Tag) -> Result<Self::Request>
    {
        Ok(0)
    }

    fn progress(&mut self, req: Self::Request) -> Result<SerdeRequestStatus> {
        Ok(SerdeRequestStatus::InProgress)
    }

    fn data<T>(&self, req: Self::Request) -> Option<T> where T: Serialize + DeserializeOwned {
        None
    }
}
