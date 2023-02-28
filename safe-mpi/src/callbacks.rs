use std::cell::Cell;
use std::os::raw::c_void;
use std::rc::Rc;
use log::info;
use ucx2_sys::{
    ucs_status_t,
    ucp_tag_recv_info_t,
    UCS_OK,
};
use crate::status_to_string;

pub extern "C" fn send_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    user_data: *mut c_void,
) {
    panic!("In send_nbx_callback with status: {}", status_to_string(status));
}

pub unsafe extern "C" fn tag_recv_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    tag_info: *const ucp_tag_recv_info_t,
    user_data: *mut c_void,
) {
    if status != UCS_OK {
        panic!("Request failed with: {}", status_to_string(status));
    }
    let done = Rc::from_raw(user_data as *const Cell<bool>);
    (*done).set(true);
    info!("Received value");
    Rc::into_raw(done);
}

pub extern "C" fn stream_recv_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    length: usize,
    user_data: *mut c_void,
) {
    panic!("In stream_recv_nbx_callback with length {} and status {}",
           length, status_to_string(status));
}
