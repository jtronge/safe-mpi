pub mod consts {
    pub use ucx2_sys::{
        UCP_PARAM_FIELD_FEATURES,
        UCS_OK,
        UCP_FEATURE_AM,
        UCP_WORKER_PARAM_FIELD_THREAD_MODE,
        UCP_EP_PARAMS_FLAGS_CLIENT_SERVER,
        UCS_THREAD_MODE_SINGLE,
        UCP_ERR_HANDLING_MODE_PEER,
        UCP_EP_PARAM_FIELD_FLAGS,
        UCP_EP_PARAM_FIELD_SOCK_ADDR,
        UCP_EP_PARAM_FIELD_ERR_HANDLER,
        UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
        UCP_EP_PARAM_FIELD_CONN_REQUEST,
        UCP_EP_CLOSE_MODE_FLUSH,
        UCP_EP_CLOSE_MODE_FORCE,
        UCP_OP_ATTR_FIELD_FLAGS,
        UCP_OP_ATTR_FIELD_CALLBACK,
        UCP_OP_ATTR_FIELD_DATATYPE,
        UCP_OP_ATTR_FIELD_USER_DATA,
        UCP_DATATYPE_CONTIG,
        UCP_DATATYPE_IOV,
        UCS_MEMORY_TYPE_HOST,
        UCS_INPROGRESS,
        UCP_LISTENER_PARAM_FIELD_SOCK_ADDR,
        UCP_LISTENER_PARAM_FIELD_CONN_HANDLER,
    };
}

// More re-exports
pub use ucx2_sys::{
    ucs_status_t,
    ucp_dt_iov_t,
    ucp_ep_h,
};
mod defaults;
pub use defaults::{
    ListenerParams,
    WorkerParams,
    EndpointParams,
    RequestParam,
};
mod ucp;
pub use ucp::{
    Context,
    Worker,
    Listener,
    Endpoint,
    Request,
    ConnRequest,
};
mod callbacks;

use ucx2_sys::{
    ucs_status_string,
};
use std::ffi::CStr;

// TODO: Don't expose ucs_status_t, instead use a wrapping Status struct.

/// Convert the status to a String.
pub fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        let ptr = ucs_status_string(status);
        CStr::from_ptr(ptr)
            .to_str()
            .expect("Could not convert pointer from ucs_status_string() into a Rust str")
            .to_string()
    }
}
