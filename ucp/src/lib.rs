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
};
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
        UCP_EP_CLOSE_MODE_FLUSH,
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
