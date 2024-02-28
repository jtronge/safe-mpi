use std::cell::RefCell;
use std::rc::Rc;
use crate::{
    request::{
        RecvIovRequest, RecvProbeRequest, Request, RequestStatus, SendIovRequest, SendRequest,
    },
    Error, Handle, Iov, MutIov, Result, Tag,
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
        Communicator { handle }
    }

    // Duplicate this communicator
    pub fn dup(&self) -> Communicator {
        Communicator {
            handle: Rc::clone(&self.handle),
        }
    }

    /// Blocking iovec send
    pub unsafe fn send(&self, data: &[Iov], tag: Tag) -> Result<usize> {
        let mut req = self.isend_iov(data, tag)?;
        while let RequestStatus::InProgress = req.progress()? {}
        req.size().ok_or(Error::InternalError)
    }

    /// Blocking recv and probe
    pub fn recv_probe(&self, tag: Tag) -> Result<Vec<u8>> {
        unsafe {
            let mut req = self.irecv_probe(tag)?;
            while let RequestStatus::InProgress = req.progress()? {}
            req.data().ok_or(Error::InternalError)
        }
    }

    /// Blocking iovec recv
    pub unsafe fn recv_iov(&self, data: &[MutIov], tag: Tag) -> Result<()> {
        let mut req = self.irecv_iov(data, tag)?;
        while let RequestStatus::InProgress = req.progress()? {}
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

    /// Non-blocking send
    pub unsafe fn isend_iov<'a>(&self, data: &'a [Iov], tag: Tag) -> Result<SendIovRequest<'a>> {
        SendIovRequest::new(Rc::clone(&self.handle), data, tag)
    }

    /// Non-blocking receive with probe
    ///
    /// This is safe, when compared with isend, since it doesn't hold any
    /// references to user-provided buffers.
    pub fn irecv_probe(&self, tag: Tag) -> Result<RecvProbeRequest> {
        Ok(RecvProbeRequest::new(Rc::clone(&self.handle), tag))
    }

    /// Non-blocking receive
    pub unsafe fn irecv_iov<'a>(&self, data: &'a [MutIov], tag: Tag) -> Result<RecvIovRequest<'a>> {
        RecvIovRequest::new(Rc::clone(&self.handle), data, tag)
    }
}
