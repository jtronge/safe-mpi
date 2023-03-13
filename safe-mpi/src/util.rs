use std::os::raw::c_void;
use log::info;
use ucx2_sys::{
    rust_ucs_ptr_status,
    rust_ucs_ptr_is_err,
    rust_ucs_ptr_is_ptr,
    ucp_worker_h,
    ucp_worker_progress,
    ucp_request_free,
    UCS_OK,
    UCS_INPROGRESS,
};
use crate::{Result, Error};

/// Wait for the request to complete
pub(crate) unsafe fn wait_loop<F>(worker: ucp_worker_h, req: *mut c_void, f: F) -> Result<()>
where
    F: Fn() -> bool,
{
    // TODO: Maybe this check should be done in the calling code
    if rust_ucs_ptr_is_ptr(req) == 0 {
        let status = rust_ucs_ptr_status(req);
        if status != UCS_OK {
            return Err(Error::FailedRequest(status));
        }
        // Already complete
        return Ok(());
    }

    if rust_ucs_ptr_is_err(req) != 0 {
        panic!("Failed to send data");
    }

    while !f() {
        info!("Waiting for request completion");
        for _ in 0..512 {
            ucp_worker_progress(worker);
        }

        let status = rust_ucs_ptr_status(req);
        if status != UCS_INPROGRESS {
            if status != UCS_OK {
                return Err(Error::FailedRequest(status));
            }
            break;
        }
    }

    ucp_request_free(req);
    Ok(())
}