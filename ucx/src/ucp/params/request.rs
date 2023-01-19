use std::convert::AsRef;
use ucx2_sys::{
    ucp_request_param_t,
    UCP_DATATYPE_CONTIG,
    UCS_MEMORY_TYPE_HOST,
    ucp_datatype_t,
    ucs_memory_type_t,
    ucp_request_param_t__bindgen_ty_1,
    ucp_request_param_t__bindgen_ty_2,
    ucp_tag_recv_info_t,
    ucs_status_t,
    // UCP_OP_ATTR_FIELD_REQUEST,
    UCP_OP_ATTR_FIELD_CALLBACK,
    UCP_OP_ATTR_FIELD_USER_DATA,
    UCP_OP_ATTR_FIELD_DATATYPE,
    UCP_OP_ATTR_FIELD_FLAGS,
    // UCP_OP_ATTR_FIELD_REPLY_BUFFER,
    UCP_OP_ATTR_FIELD_MEMORY_TYPE,
    // UCP_OP_ATTR_FIELD_RECV_INFO,
};
use crate::Status;
use crate::ucp::Request;
use crate::ucp::callbacks::{
    send_nbx_callback,
    tag_recv_nbx_callback,
    stream_and_am_recv_nbx_callback,
};
use super::InternalDefault;

#[derive(Copy, Clone)]
enum CallbackType {
    Send,
    Recv,
    RecvStream,
    RecvAM,
}

#[derive(Clone)]
pub struct RequestParam {
    inner: ucp_request_param_t,
    callback_type: Option<CallbackType>,
}

impl AsRef<ucp_request_param_t> for RequestParam {
    #[inline]
    fn as_ref(&self) -> &ucp_request_param_t {
        &self.inner
    }
}

impl Default for RequestParam {
    #[inline]
    fn default() -> Self {
        Self {
            inner: ucp_request_param_t::default(),
            callback_type: None,
        }
    }
}

impl RequestParam {
    #[inline]
    pub fn flags(mut self, flags: u32) -> Self {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_FLAGS;
        self.inner.flags = flags;
        self
    }

    // TODO: Where to free this?
    #[inline]
    pub fn cb_send<F>(mut self, f: F) -> Self
    where
        F: Fn(Request, Status),
    {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_CALLBACK
                                   | UCP_OP_ATTR_FIELD_USER_DATA;
        let f: Box<dyn Fn(Request, Status)> = Box::new(f);
        self.inner.user_data = Box::into_raw(Box::new(f)) as *mut _;
        self.inner.cb.send = Some(send_nbx_callback);
        let _ = self.callback_type.insert(CallbackType::Send);
        self
    }

    #[inline]
    pub fn cb_recv<F>(mut self, f: F) -> Self
    where
        F: Fn(Request, Status, *const ucp_tag_recv_info_t),
    {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_CALLBACK
                                   | UCP_OP_ATTR_FIELD_USER_DATA;
        let f: Box<
            dyn Fn(Request, Status, *const ucp_tag_recv_info_t)
        > = Box::new(f);
        self.inner.user_data = Box::into_raw(Box::new(f)) as *mut _;
        self.inner.cb.recv = Some(tag_recv_nbx_callback);
        let _ = self.callback_type.insert(CallbackType::Recv);
        self
    }

    #[inline]
    pub fn cb_recv_stream<F>(mut self, f: F) -> Self
    where
        F: Fn(Request, Status, usize),
    {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_CALLBACK
                                   | UCP_OP_ATTR_FIELD_USER_DATA;
        let f: Box<dyn Fn(Request, Status, usize)> = Box::new(f);
        self.inner.user_data = Box::into_raw(Box::new(f)) as *mut _;
        self.inner.cb.recv_stream = Some(stream_and_am_recv_nbx_callback);
        let _ = self.callback_type.insert(CallbackType::RecvStream);
        self
    }

    #[inline]
    pub fn cb_recv_am<F>(mut self, f: F) -> Self
    where
        F: Fn(Request, Status, usize),
    {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_CALLBACK
                                   | UCP_OP_ATTR_FIELD_USER_DATA;
        let f: Box<dyn Fn(Request, Status, usize)> = Box::new(f);
        self.inner.user_data = Box::into_raw(Box::new(f)) as *mut _;
        self.inner.cb.recv_am = Some(stream_and_am_recv_nbx_callback);
        let _ = self.callback_type.insert(CallbackType::RecvAM);
        self
    }

    #[inline]
    pub fn datatype(mut self, datatype: ucp_datatype_t) -> Self {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_DATATYPE;
        self.inner.datatype = datatype;
        self
    }

    #[inline]
    pub fn memory_type(mut self, memory_type: ucs_memory_type_t) -> Self {
        self.inner.op_attr_mask |= UCP_OP_ATTR_FIELD_MEMORY_TYPE;
        self.inner.memory_type = memory_type;
        self
    }
}

impl Drop for RequestParam {
    fn drop(&mut self) {
        if let Some(callback_type) = self.callback_type {
            unsafe {
                match callback_type {
                    CallbackType::Send => {
                        let _ = Box::from_raw(
                            self.inner.user_data as *mut Box<
                                dyn Fn(Request, ucs_status_t)
                            >
                        );
                    }
                    CallbackType::Recv => {
                        let _ = Box::from_raw(
                            self.inner.user_data as *mut Box<
                                dyn Fn(
                                    Request,
                                    ucs_status_t,
                                    *const ucp_tag_recv_info_t,
                                )
                            >
                        );
                    }
                    CallbackType::RecvStream | CallbackType::RecvAM => {
                        let _ = Box::from_raw(
                            self.inner.user_data as *mut Box<
                                dyn Fn(Request, ucs_status_t, usize)
                            >
                        );
                    }
                }
            }
        }
    }
}

impl InternalDefault for ucp_request_param_t {
    #[inline]
    fn default() -> Self {
        Self {
            op_attr_mask: 0,
            flags: 0,
            request: std::ptr::null_mut(),
            cb: ucp_request_param_t__bindgen_ty_1::default(), 
            datatype: UCP_DATATYPE_CONTIG.into(),
            user_data: std::ptr::null_mut(),
            reply_buffer: std::ptr::null_mut(),
            memory_type: UCS_MEMORY_TYPE_HOST,
            recv_info: ucp_request_param_t__bindgen_ty_2::default(),
        }
    }
}

impl InternalDefault for ucp_request_param_t__bindgen_ty_1 {
    #[inline]
    fn default() -> Self {
        Self {
            send: None,
        }
    }
}

impl InternalDefault for ucp_request_param_t__bindgen_ty_2 {
    #[inline]
    fn default() -> Self {
        ucp_request_param_t__bindgen_ty_2 {
            length: std::ptr::null_mut(),
        }
    }
}
