use log::error;
use ucx2_sys::{
    rust_ucp_init,
    ucs_status_t,
    ucp_context_h,
    ucp_params_t,
    ucs_status_string,
    UCP_PARAM_FIELD_FEATURES,
    UCP_FEATURE_TAG,
    UCP_FEATURE_STREAM,
    UCS_OK,
};
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::result::Result as StandardResult;

mod communicator;
mod context;
use context::Context;
mod request;
mod stream;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InitFailure,
}

type Result<T> = StandardResult<T, Error>;

/// Initialize the safe mpi context.
pub fn init(sockaddr: SocketAddr, server: bool) -> Result<Context> {
    // Initialize logging
    env_logger::init();
    unsafe {
        let mut context = MaybeUninit::<ucp_context_h>::uninit();
        let mut params = MaybeUninit::<ucp_params_t>::uninit().assume_init();
        params.field_mask = UCP_PARAM_FIELD_FEATURES.into();
        let features = UCP_FEATURE_TAG | UCP_FEATURE_STREAM;
        params.features = features.into();
        let status = rust_ucp_init(&params, std::ptr::null(), context.as_mut_ptr());
        if status != UCS_OK {
            error!("Failed to create context: {}", status_to_string(status));
            Err(Error::InitFailure)
        } else {
            let context = context.assume_init();
            Ok(Context::new(context, sockaddr, server))
        }
    }
}

/*
/// Wait for a request to finish.
///
/// TODO: Perhaps this done variable is not following proper safety protocols
pub(crate) unsafe fn wait_loop(
    worker: ucp_worker_h,
    req: *const c_void,
    done: &bool,
) {
    if rust_ucs_ptr_is_ptr(req) == 0 {
        let status = rust_ucs_ptr_status(req);
        if status != UCS_OK {
            panic!("Request failed: {}", status_to_string(status));
        }
        // Already done
        return;
    }
    if rust_ucs_ptr_is_err(req) != 0 {
        panic!("Failed to send data");
    }

    let mut i = 0;
    loop {
        info!("In wait loop {}", i);
        // Make some progress
        for j in 0..1024 {
            ucp_worker_progress(worker);
        }
        // Then get the status
        let status = rust_ucs_ptr_status(req);
        debug!("status: {}", status_to_string(status));
        if status != UCS_INPROGRESS {
            // Request is finished
            if status != UCS_OK {
                panic!(
                    "Request failed to complete: {}",
                    status_to_string(status),
                );
            }
            break;
        }

        // Check if the done variable is set
        if *done {
            break;
        }
        i += 1;
    }
}
*/

pub(crate) fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        CStr::from_ptr(ucs_status_string(status))
            .to_str()
            .expect("Failed to convert status string")
            .to_string()
    }
}
