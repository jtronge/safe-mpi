//! UCX context handle
use crate::{
    communicator::{self, Communicator, Message, MessageMut, Status},
    status_to_string, callbacks, Error, Handle, Iov, MutIov, Result, Tag,
    Request,
};
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use ucx2_sys::{
    rust_ucp_dt_make_contig, ucp_ep_create, ucp_ep_h, ucp_ep_params_t,
    ucp_worker_h, ucp_request_param_t, ucp_request_param_t__bindgen_ty_1, ucp_tag_send_nbx, UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS, UCP_ERR_HANDLING_MODE_PEER, UCS_OK,
    UCP_OP_ATTR_FIELD_DATATYPE, UCP_OP_ATTR_FIELD_CALLBACK, UCP_OP_ATTR_FIELD_USER_DATA,
};

/// Data reference type for send request.
pub enum Data<'a> {
    /// Contiguous data contained all in one stream.
    Contiguous(&'a [u8]),

    /// Data broken up into chunks of references.
    Chunked(&'a [&'a [u8]]),
}

pub struct Context {
    /// Handle with ucx info.
    handle: Rc<RefCell<Handle>>,
}

impl Context {
    /// Create a new context.
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Context {
        Context { handle }
    }
}

impl Communicator for Context {
    type Request = usize;

    fn size(&self) -> i32 {
        self.handle.borrow().size as i32
    }

    fn rank(&self) -> i32 {
        self.handle.borrow().rank as i32
    }

    unsafe fn isend<M: Message>(
        &self,
        data: M,
        dest: i32,
        tag: i32,
    ) -> communicator::Result<Self::Request> {
        let mut handle = self.handle.borrow_mut();
        let endpoint = handle.endpoints[dest as usize].clone();
        let datatype = rust_ucp_dt_make_contig(1);
        // Callback info
        let cb_info: *mut bool = Box::into_raw(Box::new(false));
        let param = ucp_request_param_t {
            op_attr_mask: UCP_OP_ATTR_FIELD_DATATYPE
                | UCP_OP_ATTR_FIELD_CALLBACK
                | UCP_OP_ATTR_FIELD_USER_DATA,
            datatype: datatype as u64,
            cb: ucp_request_param_t__bindgen_ty_1 {
                send: Some(callbacks::send_nbx_callback),
            },
            user_data: cb_info as *mut _,
            ..Default::default()
        };
        let request = ucp_tag_send_nbx(endpoint, data.as_ptr() as *const _, data.count(), tag as u64, &param);
        let req_id = handle.add_request(Request {
            request,
            cb_info: Some(cb_info),
        });
        Ok(req_id)
    }

    unsafe fn irecv<M: MessageMut>(
        &self,
        data: M,
        dest: i32,
        tag: i32,
    ) -> communicator::Result<Self::Request> {
        Err(communicator::Error::InternalError)
    }

    unsafe fn waitall(&self, requests: &[Self::Request]) -> communicator::Result<Vec<Status>> {
        Ok(vec![])
    }
}
