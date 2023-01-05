//! Callback wrapping code. This allows rust closures to be passed in the
//! user_data argument and then called in the different callback types.
use std::os::raw::c_void;
use ucx2_sys::{
    ucs_status_t,
    ucp_tag_recv_info_t,
    ucp_ep_h,
    ucp_conn_request_h,
};
use super::{
    Endpoint,
    Request,
    ConnRequest,
};

// ucp_request_param_t__bindgen_ty_1

/// Wrapper around ucp_send_nbx_callback_t.
pub unsafe extern "C" fn send_nbx_callback(
    request: *mut c_void,
    status: ucs_status_t,
    user_data: *mut c_void,
) {
    let req = Request::new();
    let cb = user_data as *mut Box<dyn Fn(Request, ucs_status_t)>;
    (*cb)(req, status);
}

/// Wrapper around ucp_tag_recv_nbx_callback_t.
pub unsafe extern "C" fn tag_recv_nbx_callback(
    request: *mut c_void,
    status: ucs_status_t,
    tag_info: *const ucp_tag_recv_info_t,
    user_data: *mut c_void,
) {
    let req = Request::new();
    let cb = user_data as *mut Box<
        dyn Fn(Request, ucs_status_t, *const ucp_tag_recv_info_t)
    >;
    (*cb)(req, status, tag_info);
}

/// Wrapper around both ucp_stream_recv_nbx_callback_t and
/// ucp_am_recv_data_nbx_callback_t.
pub unsafe extern "C" fn stream_and_am_recv_nbx_callback(
    request: *mut c_void,
    status: ucs_status_t,
    length: usize,
    user_data: *mut c_void,
) {
    let req = Request::new();
    let cb = user_data as *mut Box<dyn Fn(Request, ucs_status_t, usize)>;
    (*cb)(req, status, length);
}

/// Callback for ucp_listener_accept_callback_t.
pub unsafe extern "C" fn listener_accept_callback(
    ep: ucp_ep_h,
    arg: *mut c_void,
) {
    let cb = arg as *mut Box<dyn Fn(Endpoint)>;
    (*cb)(Endpoint::from_raw(ep));
}

/// Callback for ucp_listener_conn_callback_t.
pub unsafe extern "C" fn listener_conn_callback(
    conn_request: ucp_conn_request_h,
    arg: *mut c_void,
) {
    let cb = arg as *mut Box<dyn Fn(ConnRequest)>;
    (*cb)(ConnRequest::from_raw(conn_request));
}

/// Callback for ucp_err_handler_cb_t.
pub unsafe extern "C" fn err_handler_cb(
    arg: *mut c_void,
    ep: ucp_ep_h,
    status: ucs_status_t,
) {
    let cb = arg as *mut Box<dyn Fn(Endpoint, ucs_status_t)>;
    (*cb)(Endpoint::from_raw(ep), status);
}
