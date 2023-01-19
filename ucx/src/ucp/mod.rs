mod params;
pub use params::{
    ListenerParams,
    WorkerParams,
    EndpointParams,
    RequestParam,
};
mod callbacks;
mod context;
pub use context::Context;
mod worker;
pub use worker::Worker;
mod listener;
pub use listener::Listener;
mod endpoint;
pub use endpoint::Endpoint;
mod request;
pub use request::Request;
mod conn_request;
pub use conn_request::ConnRequest;

use ucx2_sys::{
    ucp_ep_close_mode,
    UCP_EP_CLOSE_MODE_FLUSH,
    UCP_EP_CLOSE_MODE_FORCE,
    ucp_ep_params_flags_field,
    UCP_EP_PARAMS_FLAGS_CLIENT_SERVER,
    UCP_EP_PARAMS_FLAGS_NO_LOOPBACK,
    UCP_EP_PARAMS_FLAGS_SEND_CLIENT_ID,
    ucp_err_handling_mode_t,
    UCP_ERR_HANDLING_MODE_NONE,
    UCP_ERR_HANDLING_MODE_PEER,
    rust_ucp_dt_make_contig,
};

pub fn make_contig(size: usize) -> usize {
    unsafe {
        rust_ucp_dt_make_contig(size)
    }
}

pub struct EPCloseMode;

impl EPCloseMode {
    pub const FLUSH: ucp_ep_close_mode = UCP_EP_CLOSE_MODE_FLUSH;
    pub const FORCE: ucp_ep_close_mode = UCP_EP_CLOSE_MODE_FORCE;
}

pub struct EPParamsFlags;

impl EPParamsFlags {
    pub const CLIENT_SERVER: ucp_ep_params_flags_field = UCP_EP_PARAMS_FLAGS_CLIENT_SERVER;
    pub const NO_LOOPBACK: ucp_ep_params_flags_field = UCP_EP_PARAMS_FLAGS_NO_LOOPBACK;
    pub const SEND_CLIENT_ID: ucp_ep_params_flags_field = UCP_EP_PARAMS_FLAGS_SEND_CLIENT_ID;
}

pub struct ErrHandlingMode;

impl ErrHandlingMode {
    pub const NONE: ucp_err_handling_mode_t = UCP_ERR_HANDLING_MODE_NONE;
    pub const PEER: ucp_err_handling_mode_t = UCP_ERR_HANDLING_MODE_PEER;
}
