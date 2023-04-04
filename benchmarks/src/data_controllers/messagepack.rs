use crate::data_controllers::{
    serde::{SerdeController, SerdeScope},
    Progress,
};
use rmp_serde;
use safe_mpi::{communicator::Communicator, Error, Iov, RequestStatus, Result, Tag};
use serde::{de::DeserializeOwned, Serialize};

pub struct MessagePackController {
    pub comm: Communicator,
}

impl MessagePackController {
    pub fn new(comm: Communicator) -> MessagePackController {
        MessagePackController { comm }
    }
}

impl SerdeController for MessagePackController {
    type Scope = MessagePackScope;

    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let buf = rmp_serde::to_vec(data).map_err(|_| Error::SerializeError)?;
            let data = [Iov(buf.as_ptr(), buf.len())];
            self.comm.send(&data, tag)
        }
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv_probe(tag)?;
        rmp_serde::decode::from_slice(&buf).map_err(|_| Error::DeserializeError)
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
    fn isend<T>(&mut self, _data: &T, _tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        Ok(0)
    }

    fn irecv(&mut self, _tag: Tag) -> Result<usize> {
        Ok(0)
    }

    fn data<T>(&self, _req: usize) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
    {
        None
    }
}

impl Progress for MessagePackScope {
    type Request = usize;

    fn progress(&mut self, _req: Self::Request) -> Result<RequestStatus> {
        Ok(RequestStatus::InProgress)
    }
}
