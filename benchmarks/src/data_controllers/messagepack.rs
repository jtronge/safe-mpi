use serde::{Serialize, de::DeserializeOwned};
use rmp_serde;
use safe_mpi::{
    Result,
    Error,
    Tag,
    RequestStatus,
    communicator::{
        Communicator,
        Data,
    },
};
use crate::data_controllers::{
    Progress,
    serde::{SerdeController, SerdeScope},
};

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
        F: FnOnce(&mut Self::Scope) -> R,
    {
        f(&mut MessagePackScope)
    }
}

pub struct MessagePackScope;

impl SerdeScope for MessagePackScope {
    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned
    {
        Ok(0)
    }

    fn irecv(&mut self, tag: Tag) -> Result<usize>
    {
        Ok(0)
    }

    fn data<T>(&self, req: usize) -> Option<T> where T: Serialize + DeserializeOwned {
        None
    }
}

impl Progress for MessagePackScope {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        Ok(RequestStatus::InProgress)
    }
}
