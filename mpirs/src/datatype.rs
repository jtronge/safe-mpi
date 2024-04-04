use ucx2_sys::{
    ucp_datatype_t,
    ucp_dt_create_generic,
    ucp_generic_dt_ops_t,
    ucs_status_t,
    UCS_OK,
};
use std::mem::MaybeUninit;
use std::ffi::c_void;
use crate::{Result, Error};
use log::debug;

/// Create a new UCX datatype.
pub unsafe fn create_datatype() -> Result<ucp_datatype_t> {
    let ops = ucp_generic_dt_ops_t {
        start_pack: Some(start_pack),
        start_unpack: Some(start_unpack),
        packed_size: Some(packed_size),
        pack: Some(pack),
        unpack: Some(unpack),
        finish: Some(finish),
    };
    let mut datatype = MaybeUninit::<ucp_datatype_t>::uninit();
    let status = ucp_dt_create_generic(&ops, std::ptr::null_mut(), datatype.as_mut_ptr());
    if status != UCS_OK {
        return Err(Error::UCXError(status));
    }
    Ok(datatype.assume_init())
}

unsafe extern "C" fn start_pack(
    context: *mut c_void,
    buffer: *const c_void,
    count: usize,
) -> *mut c_void {
    debug!("datatype::start_pack()");
    std::ptr::null_mut()
}

unsafe extern "C" fn start_unpack(
    context: *mut c_void,
    buffer: *mut c_void,
    count: usize,
) -> *mut c_void {
    debug!("datatype::start_unpack()");
    std::ptr::null_mut()
}

/// Determine the packed size of the datatype.
unsafe extern "C" fn packed_size(state: *mut c_void) -> usize {
    debug!("datatype::packed_size()");
    0
}

unsafe extern "C" fn pack(
    state: *mut c_void,
    offset: usize,
    dest: *mut c_void,
    max_length: usize,
) -> usize {
    debug!("datatype::pack()");
    0
}

unsafe extern "C" fn unpack(
    state: *mut c_void,
    offset: usize,
    src: *const c_void,
    length: usize,
) -> ucs_status_t {
    debug!("datatype::unpack()");
    UCS_OK
}

unsafe extern "C" fn finish(state: *mut c_void) {
    debug!("datatype::finish()");
}
