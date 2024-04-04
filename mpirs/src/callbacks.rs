use std::os::raw::c_void;
use ucx2_sys::{ucp_tag_recv_info_t, ucs_status_t, UCS_OK};

pub(crate) unsafe extern "C" fn send_nbx_callback(
    _req: *mut c_void,
    status: ucs_status_t,
    user_data: *mut c_void,
) {
    let cb_info = user_data as *mut bool;
    *cb_info = status == UCS_OK;
}

pub(crate) unsafe extern "C" fn tag_recv_nbx_callback(
    _req: *mut c_void,
    status: ucs_status_t,
    _tag_info: *const ucp_tag_recv_info_t,
    user_data: *mut c_void,
) {
    let done = user_data as *mut bool;
    *done = status == UCS_OK;
}
