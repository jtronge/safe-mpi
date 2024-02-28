use crate::data_controllers::{
    serde::{SerdeController, SerdeScope},
    Progress,
};
use safe_mpi::{
    communicator::{Communicator, Data},
    Error, Iov, Request as SRequest, RequestStatus, Result, Tag,
};
use serde::{de::DeserializeOwned, Serialize};
use std::os::raw::c_void;

pub struct BincodeController {
    comm: Communicator,
}

impl BincodeController {
    pub fn new(comm: Communicator) -> BincodeController {
        BincodeController { comm }
    }
}

impl SerdeController for BincodeController {
    type Scope = BincodeScope;

    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let buf = bincode::serialize(data).map_err(|_| Error::SerializeError)?;
            let data = [Iov(buf.as_ptr(), buf.len())];
            self.comm.send(&data, tag)
        }
    }

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = self.comm.recv_probe(tag)?;
        // bincode::deserialize(&buf)
        //    .map_err(|_| Error::DeserializeError)
        Ok(bincode::deserialize(&buf).unwrap())
    }

    fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Self::Scope) -> R,
    {
        f(&mut BincodeScope {
            comm: self.comm.dup(),
            requests: vec![],
        })
    }
}

struct RequestData {
    rptr: *mut c_void,
    _data: Option<Vec<u8>>,
}

pub struct BincodeScope {
    comm: Communicator,
    requests: Vec<RequestData>,
}

impl SerdeScope for BincodeScope {
    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let i = self.requests.len();
            let data = bincode::serialize(data).map_err(|_| Error::SerializeError)?;
            let data = Some(data);
            let req = self
                .comm
                .isend(Data::Contiguous(data.as_ref().unwrap()), tag)?;
            // This is valid as long as self.data[i] and self.requests[i] are
            // always freed at the same time
            let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
            let rptr = Box::into_raw(req) as *mut c_void;
            self.requests.push(RequestData { rptr, _data: data });
            Ok(i)
        }
    }

    fn irecv(&mut self, tag: Tag) -> Result<usize> {
        let i = self.requests.len();
        let req = self.comm.irecv_probe(tag)?;
        let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
        let rptr = Box::into_raw(req) as *mut c_void;
        self.requests.push(RequestData { rptr, _data: None });
        Ok(i)
    }

    fn data<T>(&self, req: usize) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let rptr = self.requests[req].rptr as *mut Box<dyn SRequest>;
            match (*rptr).data() {
                Some(data) => bincode::deserialize(&data).ok(),
                None => None,
            }
        }
    }
}

impl Progress for BincodeScope {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        unsafe {
            let rptr = self.requests[req].rptr as *mut Box<dyn SRequest>;
            Ok((*rptr).progress()?)
        }
    }
}

impl Drop for BincodeScope {
    fn drop(&mut self) {
        unsafe {
            let mut inprogress = 0;
            for req in self.requests.iter() {
                let rptr = req.rptr as *mut Box<dyn SRequest>;
                match (*rptr).progress().unwrap() {
                    RequestStatus::InProgress => inprogress += 1,
                    RequestStatus::Complete => (),
                }
            }
            if inprogress > 0 {
                panic!("{} requests are still in progress", inprogress);
            }
        }
    }
}
