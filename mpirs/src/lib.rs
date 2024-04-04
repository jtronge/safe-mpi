use log::{debug, error, info};
use serde_json;
use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::rc::Rc;
use ucx2_sys::{
    rust_ucp_init,
    rust_ucs_ptr_is_ptr,
    ucp_address_t,
    ucp_cleanup,
    ucp_context_h,
    ucp_ep_close_nb,
    ucp_ep_create,
    ucp_ep_h,
    ucp_ep_params_t,
    ucp_params_t,
    ucp_tag_t,
    ucp_request_free,
    ucp_worker_create,
    ucp_worker_destroy,
    ucp_worker_get_address,
    ucp_worker_h,
    ucp_worker_params_t,
    ucp_worker_release_address,
    ucs_status_string,
    ucs_status_t,
    UCP_EP_CLOSE_MODE_FORCE,
    UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCP_ERR_HANDLING_MODE_PEER,
    UCP_FEATURE_STREAM,
    UCP_FEATURE_TAG,
    UCP_PARAM_FIELD_FEATURES,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCS_OK,
    UCS_THREAD_MODE_SINGLE,
};

pub type Tag = ucp_tag_t;

pub mod communicator;
mod context;
use context::Context;
mod util;
use util::wait_loop;
mod callbacks;
mod request;
// pub use request::{Request, RequestStatus};
mod exchange;
mod datatype;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Initialization failure.
    InitFailure,

    /// An error was returned by the UCX library.
    UCXError(ucs_status_t),

    /// Worker creation failed.
    WorkerCreateFailed(ucs_status_t),

    /// Address exchange failed.
    WorkerAddressFailure(ucs_status_t),

    /// Request failed.
    FailedRequest(ucs_status_t),

    /// Waiting for a worker failed.
    WorkerWait(ucs_status_t),

    /// Error occurred during deserialization.
    DeserializeError,

    /// Error occurred during serialization.
    SerializeError,

    /// Timeout occured while waiting on a request.
    RequestTimeout,

    /// Internal error occurred.
    InternalError,

    /// Invalid type received in a message.
    MessageTypeMismatch,

    /// Invalid count of elements received in a message (no partial receives allowed).
    MessageCountMismatch,
}

/// Immutable iovec.
pub struct Iov(pub *const u8, pub usize);

/// Mutable iovec.
pub struct MutIov(pub *mut u8, pub usize);

/// Handle containing the internal UCP context data and other code.
pub(crate) struct Handle {
    /// UCP context.
    pub context: ucp_context_h,

    /// UCP worker.
    pub worker: ucp_worker_h,

    /// Number of processes.
    pub size: usize,

    /// Rank of this process.
    pub rank: usize,

    /// UCP endpoints.
    pub endpoints: Vec<ucp_ep_h>,

    /// Current requests.
    pub requests: Vec<Option<Request>>,

    /// Index of free requests.
    pub free_requests: Vec<usize>,
}

impl Handle {
    /// Add a new request pointer.
    pub(crate) fn add_request(&mut self, request: Request) -> usize {
        if let Some(i) = self.free_requests.pop() {
            assert!(self.requests[i].is_none());
            let _ = self.requests[i].insert(request);
            i
        } else {
            let i = self.requests.len();
            self.requests.push(Some(request));
            i
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            // Free requests.
            for req in &mut self.requests {
                if let Some(req) = req.take() {
                    drop(req);
                }
            }
            // Destroy endpoints.
            for ep in &self.endpoints {
                // For some reason UCP_EP_CLOSE_MODE_FLUSH is causing an
                // infinite loop with two nodes.
                // let req = ucp_ep_close_nb(endpoint, UCP_EP_CLOSE_MODE_FLUSH);
                let req = ucp_ep_close_nb(ep.clone(), UCP_EP_CLOSE_MODE_FORCE);
                wait_loop(self.worker, req, || false).unwrap();
            }
            ucp_worker_destroy(self.worker);
            ucp_cleanup(self.context);
        }
    }
}

/// Request struct.
pub(crate) struct Request {
    request: *mut c_void,
    cb_info: Option<*mut bool>,
}

impl Drop for Request {
    fn drop(&mut self) {
        unsafe {
            if rust_ucs_ptr_is_ptr(self.request) != 0 {
                ucp_request_free(self.request);
            }
            if let Some(cb_info) = self.cb_info {
                let _ = Box::from_raw(cb_info);
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Initialize the safe mpi context.
pub fn init() -> Result<Context> {
    // Initialize logging.
    env_logger::init();

    unsafe {
        let mut context = MaybeUninit::<ucp_context_h>::uninit();
        let params = ucp_params_t {
            field_mask: UCP_PARAM_FIELD_FEATURES.into(),
            features: (UCP_FEATURE_TAG | UCP_FEATURE_STREAM).into(),
            ..Default::default()
        };
        let status = rust_ucp_init(&params, std::ptr::null(), context.as_mut_ptr());
        if status != UCS_OK {
            error!("Failed to create context: {}", status_to_string(status));
            Err(Error::InitFailure)
        } else {
            let context = context.assume_init();
            let conn_list: Vec<String> = std::env::var("MPIRS_CONN_LIST")
                .expect("missing MPIRS_RANK in environment required for MPI initialization")
                .split(',')
                .map(|s| s.to_string())
                .collect();
            let size = conn_list.len();
            let rank: usize = std::env::var("MPIRS_RANK")
                .expect("missing MPIRS_RANK in environment required for MPI initialization")
                .parse()
                .expect("invalid rank data");
            let worker = create_worker(context)?;
            let worker_addr = get_worker_address(worker)?;
            let addrs = exchange::address_exchange(rank as usize, &conn_list, &worker_addr);
            let mut endpoints = vec![];
            for ep_rank in 0..size {
                if let Some(addr) = addrs[ep_rank].as_ref() {
                    endpoints.push(create_endpoint(worker, addr));
                } else {
                    endpoints.push(create_endpoint(worker, &worker_addr));
                }
            }
            Ok(Context::new(Rc::new(RefCell::new(Handle {
                context,
                worker,
                size,
                rank,
                endpoints,
                requests: vec![],
                free_requests: vec![],
            }))))
            /*
                        let other_addr = exchange_addrs(context, worker, server, sockaddr)?;
                        Ok(Context::new(Rc::new(RefCell::new(Handle {
                            context,
                            worker,
                            other_addr,
                            endpoint: None,
                        }))))
            */
        }
    }
}

/// Create the worker.
unsafe fn create_worker(context: ucp_context_h) -> Result<ucp_worker_h> {
    // First create the worker
    let mut worker = MaybeUninit::<ucp_worker_h>::uninit();
    let params = ucp_worker_params_t {
        field_mask: UCP_WORKER_PARAM_FIELD_THREAD_MODE.into(),
        // One thread for now
        thread_mode: UCS_THREAD_MODE_SINGLE,
        ..Default::default()
    };
    let status = ucp_worker_create(context, &params, worker.as_mut_ptr());
    if status != UCS_OK {
        Err(Error::WorkerCreateFailed(status))
    } else {
        Ok(worker.assume_init())
    }
}

/// Return the address for the worker.
unsafe fn get_worker_address(worker: ucp_worker_h) -> Result<Vec<u8>> {
    // Get the address of the worker.
    let mut addr = MaybeUninit::<*mut ucp_address_t>::uninit();
    let mut len = MaybeUninit::<usize>::uninit();
    let status = ucp_worker_get_address(worker, addr.as_mut_ptr(), len.as_mut_ptr());
    if status != UCS_OK {
        return Err(Error::WorkerAddressFailure(status));
    }
    let addr = addr.assume_init();
    let len = len.assume_init();
    let mut buffer = Vec::with_capacity(len);
    std::ptr::copy(addr as *const u8, buffer.as_mut_ptr(), len);
    buffer.set_len(len);
    Ok(buffer)
}

/// Create an endpoint for the worker and remote address.
unsafe fn create_endpoint(worker: ucp_worker_h, remote_addr: &[u8]) -> ucp_ep_h {
    let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
    let params = ucp_ep_params_t {
        field_mask: (UCP_EP_PARAM_FIELD_REMOTE_ADDRESS | UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE)
            .into(),
        err_mode: UCP_ERR_HANDLING_MODE_PEER,
        address: remote_addr.as_ptr() as *const _,
        ..Default::default()
    };
    let status = ucp_ep_create(worker, &params, endpoint.as_mut_ptr());
    if status != UCS_OK {
        panic!(
            "Failed to create endpoint for worker: {}",
            status_to_string(status)
        );
    }
    endpoint.assume_init()
}

/// Convert the ucs_status_t to a Rust string.
pub(crate) fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        CStr::from_ptr(ucs_status_string(status))
            .to_str()
            .expect("Failed to convert status string")
            .to_string()
    }
}
