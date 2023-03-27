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
use crate::data_controllers::serde::{SerdeController, SerdeScope, SerdeRequestStatus};

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

    fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Self::Scope) -> R,
    {
        f(&mut PostcardScope)
    }
}

pub struct PostcardScope;

impl SerdeScope for PostcardScope {
    type Request = usize;

    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<Self::Request>
    where
        T: Serialize + DeserializeOwned
    {
        Ok(0)
    }

    fn irecv(&mut self, tag: Tag) -> Result<Self::Request> {
        Ok(0)
    }

    fn progress(&mut self, req: Self::Request) -> Result<SerdeRequestStatus> {
        Ok(SerdeRequestStatus::InProgress)
    }

    fn data<T>(&self, req: Self::Request) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
    {
        None
    }
}
