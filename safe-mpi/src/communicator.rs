use std::rc::Rc;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use log::info;
use serde::{Serialize, de::DeserializeOwned};
use crate::{
    Result,
    Error,
    Handle,
    Iov,
    MutIov,
    Tag,
    status_to_string,
    context::Context,
    util::wait_loop,
    request::{RequestStatus, Request, RecvRequest, SendRequest}
};

/// Data reference type for send request
pub enum Data<'a> {
    /// Contiguous data contained all in one stream
    Contiguous(&'a [u8]),
    /// Data broken up into chunks of references
    Chunked(&'a [&'a [u8]]),
}

/// Communicator object providing low-level point-to-point API
pub struct Communicator {
    handle: Rc<RefCell<Handle>>,
}

impl Communicator {
    /// Create a new communicator from a handle
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Communicator {
        Communicator {
            handle,
        }
    }

    // Duplicate this communicator
    pub fn dup(&self) -> Communicator {
        Communicator {
            handle: Rc::clone(&self.handle),
        }
    }

    /// Blocking send
    pub fn send(&self, data: Data, tag: Tag) -> Result<usize> {
        unsafe {
            let mut req = self.isend(data, tag)?;
            while let RequestStatus::InProgress = req.progress()? {}
            req.size().ok_or(Error::InternalError)
        }
    }

    /// Blocking iovec send
    pub fn send_iov(&self, data: &[Iov], tag: Tag) -> Result<usize> {
        Ok(0)
    }

    /// Blocking recv
    pub fn recv(&self, tag: Tag) -> Result<Vec<u8>> {
        unsafe {
            let mut req = self.irecv(tag)?;
            while let RequestStatus::InProgress = req.progress()? {}
            req.data().ok_or(Error::InternalError)
        }
    }

    /// Blocking iovec recv
    pub fn recv_iov(&self, data: &[MutIov], tag: Tag) -> Result<()> {
        Ok(())
    }

    /// Non-blocking send
    ///
    /// This is unsafe, since if the we were to do something like
    /// `mem::forget(sreq)` the data reference would be lost and the owning code
    /// could deallocate the original data, causing a segfault sometime later
    /// when other code attempts to make progress.
    pub unsafe fn isend<'a>(&self, data: Data<'a>, tag: Tag) -> Result<SendRequest<'a>> {
        SendRequest::new(Rc::clone(&self.handle), data, tag)
    }

    /// Non-blocking receive
    ///
    /// This is safe, when compared with isend, since it doesn't hold any
    /// references to user-provided buffers.
    pub fn irecv(&self, tag: Tag) -> Result<RecvRequest> {
        Ok(RecvRequest::new(Rc::clone(&self.handle), tag))
    }
}
