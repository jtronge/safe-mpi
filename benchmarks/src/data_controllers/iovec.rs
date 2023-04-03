use std::os::raw::c_void;
use std::borrow::Borrow;
use std::marker::PhantomData;
use iovec::{ChunkSerDe, Chunk};
use safe_mpi::{
    Error,
    Iov,
    RequestStatus,
    Request as SRequest,
    Result,
    Tag,
    communicator::{
        Communicator,
        Data,
    },
};
use crate::data_controllers::Progress;

pub struct IovecController {
    pub comm: Communicator,
}

impl IovecController {
    pub fn new(comm: Communicator) -> IovecController {
        IovecController {
            comm,
        }
    }

    pub fn send<T>(&self, data: &[T], tag: Tag) -> Result<usize>
    where
        T: ChunkSerDe,
    {
        unsafe {
            let mut chunks = vec![];
            T::serialize(data, &mut chunks)
                .map_err(|_| Error::SerializeError)?;
            let send_data: Vec<Iov> = chunks
                .iter()
                .map(|chunk| match chunk {
                    Chunk::Slice(slice) => Iov(slice.as_ptr(), slice.len()),
                    Chunk::Data(data) => Iov(data.as_ptr(), data.len()),
                })
                .collect();
            self.comm.send(&send_data, tag)
        }
    }

    pub fn recv<T>(&self, tag: Tag) -> Result<Vec<T>>
    where
        T: ChunkSerDe,
    {
        let buf = self.comm.recv_probe(tag)?;
        // TODO: Should map errors to more specific message
        let (data, size) = T::deserialize(&buf)
            .map_err(|err| Error::DeserializeError)?;
        Ok(data)
    }

    /// Create a scope for running non blocking requests.
    pub fn scope<'env, F, R>(&self, f: F) -> R
    where
        F: for<'scope> FnOnce(&mut IovecScope<'scope, 'env>) -> R,
    {
        f(&mut IovecScope {
            comm: self.comm.dup(),
            requests: vec![],
            scope: PhantomData,
            env: PhantomData,
        })
    }
}

/// This is really bad, but I'm not sure if there's a better way to tell the
/// compiler that this is safe.
pub struct RequestData {
    chunks: *mut c_void,
    send_data: *mut c_void,
}

/// Request holding the pointer and any data.
pub struct Request {
    rptr: *mut c_void,
    data: Option<RequestData>
}

/// Iovec scope, based on std::thread's scope code.
pub struct IovecScope<'scope, 'env: 'scope> {
    comm: Communicator,
    requests: Vec<Request>,
    /// Invariance is over 'scope, as in std::thread
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'scope, 'env> IovecScope<'scope, 'env> {
    /// Do a non-blocking send, returning the request index.
    pub fn isend<T>(&mut self, data: &'scope [T], tag: Tag) -> Result<usize>
    where
        T: ChunkSerDe,
    {
        unsafe {
            let i = self.requests.len();
            let mut chunks = vec![];
            T::serialize(data, &mut chunks)
                .map_err(|_| Error::SerializeError)?;
            let chunks = Box::new(chunks);
            let chunks = Box::into_raw(chunks);
            let send_data: Vec<&[u8]> = (*chunks)
                .iter()
                .map(|chunk| match chunk {
                    Chunk::Slice(slice) => slice,
                    Chunk::Data(data) => &data[..],
                })
                .collect();
            let send_data = Box::new(send_data);
            let req = self.comm.isend(
                Data::Chunked(&send_data[..]),
                tag,
            )?;
            let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
            let rptr = Box::into_raw(req) as *mut c_void;
            self.requests.push(Request {
                rptr,
                data: Some(RequestData {
                    chunks: chunks as *mut c_void,
                    send_data: Box::into_raw(send_data) as *mut c_void,
                }),
            });
            Ok(i)
        }
    }

    /// Do a non-blocking receive for a type that will be deserialized later.
    /// Returns the request index.
    pub fn irecv(&mut self, tag: Tag) -> Result<usize> {
        let i = self.requests.len();
        let req = self.comm.irecv_probe(tag)?;
        let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
        let rptr = Box::into_raw(req) as *mut c_void;
        self.requests.push(Request {
            rptr,
            data: None,
        });
        Ok(i)
    }

    pub fn data<T>(&self, req: usize) -> Option<Vec<T>>
    where
        T: ChunkSerDe,
    {
        unsafe {
            let rptr = self.requests[req].rptr as *mut Box<dyn SRequest>;
            match (*rptr).data() {
                Some(data) => T::deserialize(&data)
                    .map(|(data, _)| data)
                    .ok(),
                None => None,
            }
        }
    }
}

impl<'scope, 'env> Progress for IovecScope<'scope, 'env> {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        unsafe {
            let rptr = self.requests[req].rptr as *mut Box<dyn SRequest>;
            Ok((*rptr).progress()?)
        }
    }
}

impl<'scope, 'env> Drop for IovecScope<'scope, 'env> {
    fn drop(&mut self) {
        unsafe {
            // First ensure all requests have finished
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

            // Now we can safely drop all held data
            for req in self.requests.iter() {
                if let Some(data) = &req.data {
                    let chunks = data.chunks as *mut Vec<Chunk>;
                    let chunks = Box::from_raw(chunks);
                    let send_data = data.send_data as *mut Vec<&[u8]>;
                    let send_data = Box::from_raw(send_data);
                }
            }
        }
    }
}
