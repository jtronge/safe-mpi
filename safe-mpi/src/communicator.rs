use std::rc::Rc;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use log::info;
use serde::{Serialize, de::DeserializeOwned};
use ucx2_sys::{
    rust_ucp_dt_make_contig,
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_status,
    rust_ucs_ptr_is_err,
    ucp_dt_iov,
    ucp_ep_close_nb,
    ucp_ep_h,
    ucp_request_free,
    ucp_request_param_t,
    ucs_status_t,
    ucp_tag_message_h,
    ucp_tag_msg_recv_nbx,
    ucp_tag_recv_info_t,
    ucp_tag_recv_nbx,
    ucp_tag_probe_nb,
    ucp_tag_send_nbx,
    ucp_tag_t,
    ucp_worker_h,
    ucp_worker_progress,
    ucp_worker_wait,
    UCP_DATATYPE_IOV,
    UCP_EP_CLOSE_MODE_FLUSH,
    UCP_OP_ATTR_FIELD_DATATYPE,
    UCP_OP_ATTR_FIELD_CALLBACK,
    UCP_OP_ATTR_FIELD_USER_DATA,
    UCP_OP_ATTR_FLAG_NO_IMM_CMPL,
    UCS_INPROGRESS,
    UCS_OK,
};
use crate::{
    Result,
    Error,
    Handle,
    Tag,
    status_to_string,
};
use crate::context::Context;
use crate::util::wait_loop;
use crate::callbacks::{
    send_nbx_callback,
    tag_recv_nbx_callback,
};

pub struct Communicator {
    handle: Rc<RefCell<Handle>>,
}

impl Communicator {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Communicator {
        Communicator {
            handle,
        }
    }

/*
    fn wait<R>(&self, req: &mut R) -> Result<()>
    where
        R: Request,
    {
        loop {
            // Progress the entire communicator
            self.progress()?;
            // Progress the individual request
            let status = req.progress()?;
            match status {
                RequestStatus::InProgress => continue,
                RequestStatus::Complete => break,
            }
        }
        Ok(())
    }
*/

    pub fn send(&self, data: Data, tag: Tag) -> Result<usize> {
        let mut req = self.isend(data, tag)?;
        while let RequestStatus::InProgress = req.progress()? {}
        req.size().ok_or(Error::InternalError)
    }

    pub fn recv(&self, tag: Tag) -> Result<Vec<u8>> {
        let mut req = self.irecv(tag)?;
        while let RequestStatus::InProgress = req.progress()? {}
        req.data().ok_or(Error::InternalError)
    }

    /// Do an immediate non-blocking send
    pub fn isend<'a>(&self, data: Data<'a>, tag: Tag) -> Result<SendRequest<'a>> {
        unsafe {
            let endpoint = self.handle.borrow().endpoint.unwrap();
            let (ptr, len, req_size, datatype, iov) = match &data {
                Data::Contiguous(buf) => (
                    buf.as_ptr() as *const _,
                    buf.len(),
                    buf.len(),
                    rust_ucp_dt_make_contig(1).try_into().unwrap(),
                    None,
                ),
                Data::Chunked(chunks) => {
                    let datatype = UCP_DATATYPE_IOV.try_into().unwrap();
                    let mut total = 0;
                    let iov: Option<Vec<ucp_dt_iov>> = Some(
                        chunks
                            .iter()
                            .map(|slice| {
                                total += slice.len();
                                ucp_dt_iov {
                                    buffer: slice.as_ptr() as *mut _,
                                    length: slice.len(),
                                }
                            })
                            .collect()
                    );
                    (
                        iov.as_ref().unwrap().as_ptr() as *const _,
                        chunks.len(),
                        total,
                        datatype,
                        iov,
                    )
                }
            };
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK
                                 | UCP_OP_ATTR_FIELD_USER_DATA;
            param.datatype = datatype;
            param.cb.send = Some(send_nbx_callback);
            // Callback info
            let cb_info: *mut bool = Box::into_raw(Box::new(false));
            param.user_data = cb_info as *mut _;

            let req = ucp_tag_send_nbx(
                endpoint,
                ptr,
                len,
                tag,
                &param,
            );
            Ok(SendRequest {
                complete: cb_info,
                req,
                req_size,
                handle: Rc::clone(&self.handle),
                data,
                // The iov data needs to be stored as long as the request is
                // alive
                iov,
            })
        }
    }

    pub fn irecv(&self, tag: Tag) -> Result<RecvRequest> {
        let complete = Box::new(false);
        let complete = Box::into_raw(complete);
        Ok(RecvRequest {
            state: RecvRequestState::Probe,
            tag,
            complete,
            req: std::ptr::null_mut(),
            handle: Rc::clone(&self.handle),
            data: None,
        })
    }

/*
    pub fn progress(&self) -> Result<()> {
        unsafe {
            ucp_worker_progress(self.handle.borrow().worker);
            Ok(())
        }
    }
*/
}

/// Status for a communication request.
pub enum RequestStatus {
    /// Request still in progress
    InProgress,
    /// Request is now complete
    Complete,
}

pub trait Request {
    /// Progress the request
    fn progress(&mut self) -> Result<RequestStatus>;
    /// Return the request size, if it has one
    fn size(&self) -> Option<usize>;
    /// Consume the request and return received data, if there was any
    fn data(self) -> Option<Vec<u8>>;
}

pub struct SendRequest<'a> {
    /// Boolean indicating completion (allocated with Box)
    complete: *mut bool,
    req: *mut c_void,
    /// Amount of data sent in the request (in bytes)
    req_size: usize,
    /// Handle to ucx objects
    handle: Rc<RefCell<Handle>>,
    /// Data reference
    data: Data<'a>,
    /// iovecs, if used for this request
    iov: Option<Vec<ucp_dt_iov>>,
}

impl<'a> Drop for SendRequest<'a> {
    fn drop(&mut self) {
        unsafe {
            if rust_ucs_ptr_is_ptr(self.req) != 0 {
                ucp_request_free(self.req);
            }
            let _ = Box::from_raw(self.complete);
        }
    }
}

/// Progress the request and return whether it completed or not.
unsafe fn request_progress(
    worker: ucp_worker_h,
    req: *mut c_void,
    complete: *mut bool,
) -> Result<RequestStatus> {
    ucp_worker_progress(worker);

    if *complete {
        return Ok(RequestStatus::Complete);
    }

    if rust_ucs_ptr_is_ptr(req) == 0 {
        let status = rust_ucs_ptr_status(req);
        if status != UCS_OK {
            return Err(Error::FailedRequest(status));
        }
        *complete = true;
        return Ok(RequestStatus::Complete);
    }

    if rust_ucs_ptr_is_err(req) != 0 {
        return Err(Error::FailedRequest(rust_ucs_ptr_status(req)));
    }

    let status = rust_ucs_ptr_status(req);
    if status == UCS_INPROGRESS {
        Ok(RequestStatus::InProgress)
    } else {
        if status != UCS_OK {
            return Err(Error::FailedRequest(status));
        }
        Ok(RequestStatus::Complete)
    }
}

impl<'a> Request for SendRequest<'a> {
    /// Make progress on the send request
    fn progress(&mut self) -> Result<RequestStatus> {
        unsafe {
            info!("Running progress() on SendRequest");
            let worker = self.handle.borrow().worker;
            request_progress(worker, self.req, self.complete)
        }
    }

    /// Return the size of the send request
    fn size(&self) -> Option<usize> {
        Some(self.req_size)
    }

    /// Returns none, no data to return for a send request
    fn data(self) -> Option<Vec<u8>> {
        None
    }
}

enum RecvRequestState {
    /// Probing for the message
    Probe,
    /// Need to wait on the message
    Wait,
    /// Request is complete
    Complete,
}

pub struct RecvRequest {
    state: RecvRequestState,
    tag: Tag,
    complete: *mut bool,
    req: *mut c_void,
    handle: Rc<RefCell<Handle>>,
    data: Option<Vec<u8>>,
}

impl Drop for RecvRequest {
    fn drop(&mut self) {
        unsafe {
            if rust_ucs_ptr_is_ptr(self.req) != 0 {
                ucp_request_free(self.req);
            }
            let _ = Box::from_raw(self.complete);
        }
    }
}

impl Request for RecvRequest {
    /// Progress the request. This will need to be called multiple times until
    /// error or `Ok(RequestStatus::Complete)` is returned.
    fn progress(&mut self) -> Result<RequestStatus> {
        unsafe {
            let worker = self.handle.borrow().worker;
            match self.state {
                RecvRequestState::Probe => {
                    ucp_worker_progress(worker);
                    let mut info = MaybeUninit::<ucp_tag_recv_info_t>::uninit();
                    // Probe for the message
                    let message = ucp_tag_probe_nb(worker, self.tag, 0, 1, info.as_mut_ptr());
                    if message != std::ptr::null_mut() {
                        // Message probed, go ahead and allocate everything and
                        // start the receive.
                        self.state = RecvRequestState::Wait;
                        let info = info.assume_init();
                        self.data.insert(vec![0; info.length]);
                        let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
                        param.op_attr_mask = UCP_OP_ATTR_FIELD_CALLBACK
                                             | UCP_OP_ATTR_FIELD_DATATYPE
                                             | UCP_OP_ATTR_FIELD_USER_DATA
                                             | UCP_OP_ATTR_FLAG_NO_IMM_CMPL;
                        param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
                        param.cb.recv = Some(tag_recv_nbx_callback);
                        param.user_data = self.complete as *mut _;
                        self.req = ucp_tag_msg_recv_nbx(
                            worker,
                            self.data.as_mut().unwrap().as_mut_ptr() as *mut _,
                            info.length,
                            message,
                            &param,
                        );
                    }
                    Ok(RequestStatus::InProgress)
                }
                RecvRequestState::Wait => {
                    // Wait until request completion
                    let worker = self.handle.borrow().worker;
                    match request_progress(worker, self.req, self.complete)? {
                        RequestStatus::Complete => {
                            self.state = RecvRequestState::Complete;
                            Ok(RequestStatus::Complete)
                        }
                        status => Ok(status),
                    }
                }
                RecvRequestState::Complete => Ok(RequestStatus::Complete),
            }
        }
    }

    /// Return the size of the data in the request.
    fn size(&self) -> Option<usize> {
        self.data.as_ref().map(|v| v.len())
    }

    /// Return the request data.
    fn data(mut self) -> Option<Vec<u8>> {
        self.data.take()
    }
}

pub enum Data<'a> {
    /// Contiguous data contained all in one stream
    Contiguous(&'a [u8]),
    /// Data broken up into chunks of references
    Chunked(&'a [&'a [u8]]),
}
