use std::convert::AsRef;
use ucx2_sys::{
    ucp_request_param_t,
    UCP_DATATYPE_CONTIG,
    UCS_MEMORY_TYPE_HOST,
    ucp_datatype_t,
    ucs_memory_type_t,
    ucp_request_param_t__bindgen_ty_1,
    ucp_request_param_t__bindgen_ty_2,
    ucp_send_nbx_callback_t,
    ucp_tag_recv_nbx_callback_t,
    ucp_stream_recv_nbx_callback_t,
    ucp_am_recv_data_nbx_callback_t,
};
use super::InternalDefault;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct RequestParam {
    inner: ucp_request_param_t,
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
        }
    }
}

impl RequestParam {
    #[inline]
    pub fn op_attr_mask(mut self, op_attr_mask: u32) -> Self {
        self.inner.op_attr_mask = op_attr_mask;
        self
    }

    #[inline]
    pub fn flags(mut self, flags: u32) -> Self {
        self.inner.flags = flags;
        self
    }

    #[inline]
    pub fn cb_send(mut self, cb: ucp_send_nbx_callback_t) -> Self {
        self.inner.cb.send = cb;
        self
    }

    #[inline]
    pub fn cb_recv(mut self, cb: ucp_tag_recv_nbx_callback_t) -> Self {
        self.inner.cb.recv = cb;
        self
    }

    #[inline]
    pub fn cb_recv_stream(mut self, cb: ucp_stream_recv_nbx_callback_t) -> Self {
        self.inner.cb.recv_stream = cb;
        self
    }

    #[inline]
    pub fn cb_recv_am(mut self, cb: ucp_am_recv_data_nbx_callback_t) -> Self {
        self.inner.cb.recv_am = cb;
        self
    }

    #[inline]
    pub fn datatype(mut self, datatype: ucp_datatype_t) -> Self {
        self.inner.datatype = datatype;
        self
    }

    #[inline]
    pub fn memory_type(mut self, memory_type: ucs_memory_type_t) -> Self {
        self.inner.memory_type = memory_type;
        self
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
