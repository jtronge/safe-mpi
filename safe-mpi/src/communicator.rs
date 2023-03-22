use std::rc::Rc;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use serde::{Serialize, de::DeserializeOwned};
use ucx2_sys::{
    rust_ucp_dt_make_contig,
    ucp_dt_iov,
    ucp_ep_close_nb,
    ucp_ep_h,
    ucp_request_free,
    ucp_request_param_t,
    ucs_status_t,
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

pub fn send(worker: ucp_worker_h, endpoint: ucp_ep_h, tag: Tag, data: Data) -> Result<usize> {
    unsafe {
        let (ptr, len, total, datatype, _iov) = match &data {
            Data::Contiguous(buf) => (
                buf.as_ptr() as *const _,
                buf.len(),
                buf.len(),
                rust_ucp_dt_make_contig(1).try_into().unwrap(),
                vec![],
            ),
            Data::Chunked(chunks) => {
                let datatype = UCP_DATATYPE_IOV.try_into().unwrap();
                let mut total = 0;
                let iov: Vec<ucp_dt_iov> = chunks
                    .iter()
                    .map(|slice| {
                        total += slice.len();
                        ucp_dt_iov {
                            buffer: slice.as_ptr() as *mut _,
                            length: slice.len(),
                        }
                    })
                    .collect();
                (iov.as_ptr() as *const _, chunks.len(), total, datatype, iov)
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
        wait_loop(worker, req, || *cb_info).unwrap();

        let _ = Box::from_raw(cb_info);
        Ok(total)
    }
}


pub fn recv(worker: ucp_worker_h, tag: ucp_tag_t, buf: &mut [u8]) -> Result<usize> {
    unsafe {
        let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
        param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                             | UCP_OP_ATTR_FIELD_CALLBACK
                             | UCP_OP_ATTR_FIELD_USER_DATA;
        param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
        param.cb.recv = Some(tag_recv_nbx_callback);
        // Callback info
        let cb_info: *mut bool = Box::into_raw(Box::new(false));
        param.user_data = cb_info as *mut _;
        let req = ucp_tag_recv_nbx(
            worker,
            buf.as_mut_ptr() as *mut _,
            buf.len(),
            tag,
            0,
            &param,
        );
        wait_loop(worker, req, || *cb_info).unwrap();
        let _ = Box::from_raw(cb_info);
        Ok(buf.len())
    }
}

impl Communicator {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Communicator {
        Communicator {
            handle,
        }
    }

    pub fn send(&self, data: Data, tag: Tag) -> Result<usize> {
        let worker = self.handle.borrow().worker;
        let endpoint = self.handle.borrow().endpoint.unwrap();
        send(worker, endpoint, tag, data)
    }

    pub fn recv(&self, tag: Tag) -> Result<Vec<u8>> {
        unsafe {
            let mut info = MaybeUninit::<ucp_tag_recv_info_t>::uninit();
            let worker = self.handle.borrow().worker;
            let mut msg;
            loop {
                // Make sure to call ucp_worker_progress first, otherwise bad
                // things will happen
                ucp_worker_progress(worker);
                msg = ucp_tag_probe_nb(worker, tag, 0, 1, info.as_mut_ptr());
                if msg != std::ptr::null_mut() {
                    break;
                }
            }
            let info = info.assume_init();
            let mut buf = vec![0; info.length];

            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_CALLBACK
                                 | UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_USER_DATA
                                 | UCP_OP_ATTR_FLAG_NO_IMM_CMPL;
            param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
            param.cb.recv = Some(tag_recv_nbx_callback);
            let cb_info = Box::into_raw(Box::new(false));
            param.user_data = cb_info as *mut _;
            let req = ucp_tag_msg_recv_nbx(worker, buf.as_mut_ptr() as *mut _, info.length, msg, &param);
            wait_loop(worker, req, || *cb_info).unwrap();
            let _ = Box::from_raw(cb_info);
            Ok(buf)
        }
    }

    pub fn isend<'a>(&self, data: Data<'a>, tag: Tag) -> Result<Request<'a>> {
        Ok(Request {
            complete: false,
            req: std::ptr::null_mut(),
            handle: Rc::clone(&self.handle),
            data: Some(data),
        })
    }

    pub fn irecv(&self, tag: Tag) -> Result<Request> {
        Ok(Request {
            complete: false,
            req: std::ptr::null_mut(),
            handle: Rc::clone(&self.handle),
            data: None,
        })
    }
}

pub struct Request<'a> {
    complete: bool,
    req: *mut c_void,
    handle: Rc<RefCell<Handle>>,
    data: Option<Data<'a>>,
}

impl<'a> Request<'a> {
/*
    pub fn wait() -> {
        let worker = self.handle.borrow().worker;
        wait_loop(worker, req, || complete).unwrap();
    }
*/
}

pub enum Data<'a> {
    /// Contiguous data contained all in one stream
    Contiguous(&'a [u8]),
    /// Data broken up into chunks of references
    Chunked(&'a [&'a [u8]]),
}
