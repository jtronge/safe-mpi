use ucx2_sys::{
    rust_ucp_init,
    ucp_cleanup,
    ucp_worker_create,
    ucs_status_string,
    ucp_ep_create,
    ucp_context_h,
    ucp_worker_h,
    ucp_ep_h,
    ucp_params_t,
    ucp_worker_params_t,
    ucp_ep_params_t,
    ucs_status_t,
    UCP_PARAM_FIELD_FEATURES,
    UCP_FEATURE_TAG,
    UCP_FEATURE_STREAM,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCS_THREAD_MODE_SINGLE,
    // UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCP_EP_PARAM_FIELD_SOCK_ADDR,
    UCS_OK,
};
use nix::sys::socket::{
    SockaddrIn,
    SockaddrLike,
};
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;
use std::result::Result as StandardResult;
use std::str::FromStr;

/// Default port to communicate on for now
const PORT: u16 = 5588;

pub struct Context {
    /// UCP context
    context: ucp_context_h,
    /// Address of other process
    address: SockaddrIn,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InitFailure,
}

type Result<T> = StandardResult<T, Error>;

/// Initialize the safe mpi context.
pub fn init(address: &str) -> Result<Context> {
    unsafe {
        let mut context = MaybeUninit::<ucp_context_h>::uninit();
        let mut params = MaybeUninit::<ucp_params_t>::uninit().assume_init();
        params.field_mask = UCP_PARAM_FIELD_FEATURES.into();
        let features = UCP_FEATURE_TAG | UCP_FEATURE_STREAM;
        params.features = features.into();
        let status = rust_ucp_init(&params, std::ptr::null(), context.as_mut_ptr());
        if status != UCS_OK {
            eprintln!("Failed to create context: {}", status_to_string(status));
            Err(Error::InitFailure)
        } else {
            let context = context.assume_init();
            // Convert the address to a SockaddrIn
            let address = Ipv4Addr::from_str(address)
                .expect("Failed to convert address to SockaddrIn");
            let ip = address.octets();
            let address = SockaddrIn::new(ip[0], ip[1], ip[2], ip[3], PORT);
            Ok(Context {
                context,
                address,
            })
        }
    }
}

impl Context {
    /// Return the world communicator.
    pub fn world<'a>(&'a self) -> Communicator<'a> {
        unsafe {
            // First create the worker
            let mut worker = MaybeUninit::<ucp_worker_h>::uninit();
            let mut params = MaybeUninit::<ucp_worker_params_t>::uninit().assume_init();
            params.field_mask = UCP_WORKER_PARAM_FIELD_THREAD_MODE.into();
            // One thread for now
            params.thread_mode = UCS_THREAD_MODE_SINGLE;
            let status = ucp_worker_create(self.context, &params, worker.as_mut_ptr());
            if status != UCS_OK {
                panic!("Failed to create ucp worker: {}", status_to_string(status));
            }
            let worker = worker.assume_init();

            // Now create the single endpoint (this will change for multiple processes)
            let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
            let mut params = MaybeUninit::<ucp_ep_params_t>::uninit().assume_init();
            params.field_mask = UCP_EP_PARAM_FIELD_SOCK_ADDR.into();
            params.sockaddr.addr = self.address.as_ptr() as *const _;
            params.sockaddr.addrlen = self.address.len();
            // TODO: other code uses the address field, what's the difference
            // between this and the sockaddr field? Does it matter?
            // params.field_mask = UCP_EP_PARAM_FIELD_REMOTE_ADDRESS.into();
            // params.address = self.address.
            let status = ucp_ep_create(worker, &params, endpoint.as_mut_ptr());
            if status != UCS_OK {
                panic!("Failed to create endpoint for worker: {}", status_to_string(status));
            }
            let endpoint = endpoint.assume_init();

            Communicator {
                context: self,
                worker,
                endpoint,
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ucp_cleanup(self.context);
        }
    }
}

pub struct Communicator<'a> {
    /// Save a reference to the context
    context: &'a Context,
    /// One worker per communicator
    worker: ucp_worker_h,
    /// Endpoint to other process (in a multi-process scenario there would be
    /// multiple endpoints here)
    endpoint: ucp_ep_h,
}

pub(crate) fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        CStr::from_ptr(ucs_status_string(status))
            .to_str()
            .expect("Failed to convert status string")
            .to_string()
    }
}
