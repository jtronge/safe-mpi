//! Data controller for types that implement FlatBuffer.
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use safe_mpi::{
    Result,
    RequestStatus,
    Request as SRequest,
    Tag,
    Error,
    Iov,
    MutIov,
    communicator::Communicator,
};
use flat::FlatBuffer;
use crate::data_controllers::Progress;

pub struct FlatController {
    pub comm: Communicator,
}

impl FlatController {
    pub fn new(comm: Communicator) -> FlatController {
        FlatController {
            comm,
        }
    }

    /// Send data from the buffer.
    pub fn send<T: ?Sized>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        unsafe {
            let type_id = <T as FlatBuffer>::type_id();
            let type_id_ptr = (&type_id as *const u64) as *const u8;
            let count = data.count();
            let count_ptr = (&count as *const usize) as *const u8;
            let iovecs = vec![
                Iov(type_id_ptr, std::mem::size_of::<u64>()),
                Iov(count_ptr, std::mem::size_of::<usize>()),
                Iov(data.ptr(), data.size()),
            ];
            self.comm.send(&iovecs[..], tag)
        }
    }

    /// Receive data into the buffer.
    pub fn recv<T: ?Sized>(&self, data: &mut T, tag: Tag) -> Result<()>
    where
        T: FlatBuffer,
    {
        unsafe {
            let mut type_id = MaybeUninit::<u64>::uninit();
            let mut count = MaybeUninit::<usize>::uninit();
            let iovecs = vec![
                MutIov(type_id.as_mut_ptr() as *mut _, std::mem::size_of::<u64>()),
                MutIov(count.as_mut_ptr() as *mut _, std::mem::size_of::<usize>()),
                MutIov(data.ptr_mut(), data.size()),
            ];
            self.comm.recv_iov(&iovecs[..], tag)?;
            let type_id = type_id.assume_init();
            let count = count.assume_init();
            if type_id != <T as FlatBuffer>::type_id() {
                Err(Error::MessageTypeMismatch)
            } else if count != data.count() {
                Err(Error::MessageCountMismatch)
            } else {
                Ok(())
            }
        }
    }

    /// Scope for non blocking requests.
    pub fn scope<'env, F, R>(&self, f: F) -> R
    where
        F: for<'scope> FnOnce(&mut FlatScope<'scope, 'env>) -> R,
    {
        f(&mut FlatScope {
            comm: self.comm.dup(),
            requests: vec![],
            scope: PhantomData,
            env: PhantomData,
        })
    }
}

struct Request {
    rptr: *mut c_void,
    type_id: *mut u64,
    count: *mut usize,
    expected_type_id: Option<u64>,
    expected_count: Option<usize>,
}

pub struct FlatScope<'scope, 'env: 'scope> {
    comm: Communicator,
    requests: Vec<Request>,
    /// Invariance is over 'scope, as in iovec.rs (borrowed from std::thread)
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'scope, 'env> FlatScope<'scope, 'env> {
    /// Do a non-blocking send, returning the request index.
    pub fn isend<T: ?Sized>(&mut self, data: &'scope T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        unsafe {
            let i = self.requests.len();
            // Prepare the iovec
            let type_id = Box::new(<T as FlatBuffer>::type_id());
            let type_id = Box::into_raw(type_id);
            let count = Box::new(data.count());
            let count = Box::into_raw(count);
            let iovecs = vec![
                Iov(type_id as *const u8, std::mem::size_of::<u64>()),
                Iov(count as *const u8, std::mem::size_of::<usize>()),
                Iov(data.ptr(), data.size()),
            ];
            let req = self.comm.isend_iov(
                &iovecs,
                tag,
            )?;
            let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
            let rptr = Box::into_raw(req) as *mut c_void;
            self.requests.push(Request {
                rptr,
                type_id,
                count,
                expected_type_id: None,
                expected_count: None,
            });
            Ok(i)
        }
    }

    /// Do a non-blocking receive, returning the request index.
    pub fn irecv<T: ?Sized>(&mut self, data: &'scope mut T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        unsafe {
            let i = self.requests.len();
            let mut type_id = Box::new(0u64);
            let type_id = Box::into_raw(type_id);
            let mut count = Box::new(0usize);
            let count = Box::into_raw(count);
            let iovecs = vec![
                MutIov(type_id as *mut _, std::mem::size_of::<u64>()),
                MutIov(count as *mut _, std::mem::size_of::<usize>()),
                MutIov(data.ptr_mut(), data.size()),
            ];
            let req = self.comm.irecv_iov(&iovecs, tag)?;
            let req: Box<Box<dyn SRequest>> = Box::new(Box::new(req));
            let rptr = Box::into_raw(req) as *mut c_void;
            self.requests.push(Request {
                rptr,
                type_id,
                count,
                expected_type_id: Some(<T as FlatBuffer>::type_id()),
                expected_count: Some(data.count()),
            });
            Ok(i)
        }
    }
}

/// Check the type ID and count of a complete request.
unsafe fn check_type_count(request: &Request) -> Result<()> {
    if let Some(expected_type_id) = request.expected_type_id {
        if (*request.type_id) != expected_type_id {
            return Err(Error::MessageTypeMismatch);
        }
    }
    if let Some(expected_count) = request.expected_count {
        if (*request.count) != expected_count {
            return Err(Error::MessageCountMismatch);
        }
    }
    Ok(())
}

impl<'scope, 'env> Progress for FlatScope<'scope, 'env> {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        unsafe {
            let rptr = self.requests[req].rptr as *mut Box<dyn SRequest>;
            match (*rptr).progress()? {
                RequestStatus::InProgress => Ok(RequestStatus::InProgress),
                RequestStatus::Complete => {
                    check_type_count(&self.requests[req])?;
                    Ok(RequestStatus::Complete)
                }
            }
        }
    }
}

impl<'scope, 'env> Drop for FlatScope<'scope, 'env> {
    fn drop(&mut self) {
        unsafe {
            // Ensure all requests have finished
            let mut inprogress = 0;
            for req in self.requests.iter() {
                let rptr = req.rptr as *mut Box<dyn SRequest>;
                match (*rptr).progress().unwrap() {
                    RequestStatus::InProgress => inprogress += 1,
                    RequestStatus::Complete => {
                        // Make sure to check the type and count
                        check_type_count(req).unwrap();
                    }
                }
            }
            if inprogress > 0 {
                panic!("{} requests are still in progress", inprogress);
            }

            // Free up attached data
            for req in self.requests.iter() {
                let _ = Box::from_raw(req.type_id);
                let _ = Box::from_raw(req.count);
            }
        }
    }
}
