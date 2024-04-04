use crate::{Error, Result};
use log::info;
use std::os::raw::c_void;
use ucx2_sys::{
    rust_ucs_ptr_is_err, rust_ucs_ptr_is_ptr, rust_ucs_ptr_status, ucp_request_free, ucp_worker_h,
    ucp_worker_progress, UCS_INPROGRESS, UCS_OK,
};

const TIMEOUT: usize = 8192;

/// Wait for the request to complete.
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

    let mut i = 0;
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
        if i > TIMEOUT {
            return Err(Error::RequestTimeout);
        }
        i += 1;
    }

    ucp_request_free(req);
    Ok(())
}
