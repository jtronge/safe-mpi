use serde::{Serialize, de::DeserializeOwned};
use postcard;
use safe_mpi::{
    Result,
    Error,
    Tag,
    RequestStatus,
    Iov,
    communicator::{
        Communicator,
        Data,
    },
};
use crate::data_controllers::{
    Progress,
    serde::{SerdeController, SerdeScope},
};

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
    type Scope = PostcardScope;

    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let buf = postcard::to_allocvec(data)
                .map_err(|_| Error::SerializeError)?;
            let data = [Iov(buf.as_ptr() as *const _, buf.len())];
            self.comm.send(&data, tag)
        }
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv_probe(tag)?;
        postcard::from_bytes(&buf)
            .map_err(|_| Error::DeserializeError)
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
    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned
    {
        Ok(0)
    }

    fn irecv(&mut self, tag: Tag) -> Result<usize> {
        Ok(0)
    }

    fn data<T>(&self, req: usize) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
    {
        None
    }
}

impl Progress for PostcardScope {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        Ok(RequestStatus::InProgress)
    }
}
