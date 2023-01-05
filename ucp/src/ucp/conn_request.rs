use ucx2_sys::{
    ucp_conn_request_h,
};

#[repr(transparent)]
pub struct ConnRequest(ucp_conn_request_h);

impl ConnRequest {
    pub fn from_raw(conn_request: ucp_conn_request_h) -> ConnRequest {
        ConnRequest(conn_request)
    }

    pub fn into_raw(&self) -> ucp_conn_request_h {
        self.0
    }
}
