use crate::data_controllers::{
    serde::{SerdeController, SerdeScope},
    Progress,
};
use postcard;
use safe_mpi::{communicator::Communicator, Error, Iov, RequestStatus, Result, Tag};
use serde::{de::DeserializeOwned, Serialize};

pub struct PostcardController {
    pub comm: Communicator,
}

impl PostcardController {
    pub fn new(comm: Communicator) -> PostcardController {
        PostcardController { comm }
    }
}

impl SerdeController for PostcardController {
    type Scope = PostcardScope;

    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let buf = postcard::to_allocvec(data).map_err(|_| Error::SerializeError)?;
            let data = [Iov(buf.as_ptr() as *const _, buf.len())];
            self.comm.send(&data, tag)
        }
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv_probe(tag)?;
        postcard::from_bytes(&buf).map_err(|_| Error::DeserializeError)
    }

    fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Self::Scope) -> R,
    {
        f(&mut PostcardScope)
    }
}

pub struct PostcardScope;

impl SerdeScope for PostcardScope {
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

impl Progress for PostcardScope {
    type Request = usize;

    fn progress(&mut self, _req: Self::Request) -> Result<RequestStatus> {
        Ok(RequestStatus::InProgress)
    }
}
