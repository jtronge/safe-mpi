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
    println!("send_nbx_callback()");
    let req = Request::new();
    let ptr = user_data as *mut Box<dyn Fn(Request, ucs_status_t)>;
    let cb = Box::from_raw(ptr);
    println!("before calling send closure");
    cb(req, status);
    let _ = Box::into_raw(cb);
    println!("after calling send closure");
}

/// Wrapper around ucp_tag_recv_nbx_callback_t.
pub unsafe extern "C" fn tag_recv_nbx_callback(
    request: *mut c_void,
    status: ucs_status_t,
    tag_info: *const ucp_tag_recv_info_t,
    user_data: *mut c_void,
) {
    println!("tag_recv_nbx_callback()");
    let req = Request::new();
    let ptr = user_data as *mut Box<
        dyn Fn(Request, ucs_status_t, *const ucp_tag_recv_info_t)
    >;
    let cb = Box::from_raw(ptr);
    cb(req, status, tag_info);
    let _ = Box::into_raw(cb);
}

/// Wrapper around both ucp_stream_recv_nbx_callback_t and
/// ucp_am_recv_data_nbx_callback_t.
pub unsafe extern "C" fn stream_and_am_recv_nbx_callback(
    request: *mut c_void,
    status: ucs_status_t,
    length: usize,
    user_data: *mut c_void,
) {
    println!("stream_and_am_recv_nbx_callback()");
    let req = Request::new();
    let ptr = user_data as *mut Box<dyn Fn(Request, ucs_status_t, usize)>;
    let cb = Box::from_raw(ptr);
    cb(req, status, length);
    let _ = Box::into_raw(cb);
}

/// Callback for ucp_listener_accept_callback_t.
pub unsafe extern "C" fn listener_accept_callback(
    ep: ucp_ep_h,
    arg: *mut c_void,
) {
    println!("listener_accept_callback()");
    let ptr = arg as *mut Box<dyn Fn(Endpoint)>;
    let cb = Box::from_raw(ptr);
    cb(Endpoint::from_raw(ep));
    let _ = Box::into_raw(cb);
}

/// Callback for ucp_listener_conn_callback_t.
pub unsafe extern "C" fn listener_conn_callback(
    conn_request: ucp_conn_request_h,
    arg: *mut c_void,
) {
    println!("listener_conn_callback()");
    let ptr = arg as *mut Box<dyn Fn(ConnRequest)>;
    let cb = Box::from_raw(ptr);
    cb(ConnRequest::from_raw(conn_request));
    let _ = Box::into_raw(cb);
}

/// Callback for ucp_err_handler_cb_t.
pub unsafe extern "C" fn err_handler_cb(
    arg: *mut c_void,
    ep: ucp_ep_h,
    status: ucs_status_t,
) {
    println!("err_handler_cb()");
    let ptr = arg as *mut Box<dyn Fn(Endpoint, ucs_status_t)>;
    let cb = Box::from_raw(ptr);
    cb(Endpoint::from_raw(ep), status);
    let _ = Box::into_raw(cb);
}
