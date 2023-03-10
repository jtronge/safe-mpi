use log::{error, info, debug};
use ucx2_sys::{
    rust_ucp_init,
    ucp_address_t,
    ucp_cleanup,
    ucp_context_h,
    ucp_ep_h,
    ucp_ep_close_nb,
    ucp_params_t,
    ucp_tag_t,
    ucp_worker_h,
    ucp_worker_create,
    ucp_worker_destroy,
    ucp_worker_get_address,
    ucp_worker_params_t,
    ucp_worker_release_address,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    ucs_status_t,
    ucs_status_string,
    UCP_EP_CLOSE_MODE_FLUSH,
    UCP_FEATURE_TAG,
    UCP_FEATURE_STREAM,
    UCP_PARAM_FIELD_FEATURES,
    UCS_THREAD_MODE_SINGLE,
    UCS_OK,
};
use std::ffi::CStr;
use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{
    TcpListener,
    TcpStream,
    SocketAddr,
    Shutdown,
    Ipv4Addr,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::result::Result as StandardResult;
use serde_json;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IPv4 address of other process
    pub address: Ipv4Addr,
    /// TCP port of other process
    pub port: u16,
    /// Is this the server process?
    #[arg(short, long)]
    pub server: bool,
}

pub type Tag = ucp_tag_t;

mod communicator;
mod context;
use context::Context;
pub mod data_controllers;
// mod request;
// mod stream;
mod util;
use util::wait_loop;
mod callbacks;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InitFailure,
    WorkerCreateFailed(ucs_status_t),
    WorkerAddressFailure(ucs_status_t),
    FailedRequest(ucs_status_t),
    WorkerWait(ucs_status_t),
    DeserializeError,
    SerializeError,
}

/// Handle containing the internal UCP context data and other code.
pub(crate) struct Handle {
    pub context: ucp_context_h,
    pub worker: ucp_worker_h,
    pub other_addr: Vec<u8>,
    pub endpoint: Option<ucp_ep_h>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            if let Some(endpoint) = self.endpoint {
                let req = ucp_ep_close_nb(endpoint, UCP_EP_CLOSE_MODE_FLUSH);
                wait_loop(self.worker, req, || false).unwrap();
            }
            ucp_worker_destroy(self.worker);
            ucp_cleanup(self.context);
        }
    }
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
            let worker = create_worker(context)?;
            let other_addr = exchange_addrs(context, worker, server, sockaddr)?;
            Ok(Context::new(
                Rc::new(RefCell::new(Handle {
                    context,
                    worker,
                    other_addr,
                    endpoint: None,
                })),
            ))
        }
    }
}

/// Create the worker.
unsafe fn create_worker(context: ucp_context_h) -> Result<ucp_worker_h> {
    // First create the worker
    let mut worker = MaybeUninit::<ucp_worker_h>::uninit();
    let mut params = MaybeUninit::<ucp_worker_params_t>::uninit().assume_init();
    params.field_mask = UCP_WORKER_PARAM_FIELD_THREAD_MODE.into();
    // One thread for now
    params.thread_mode = UCS_THREAD_MODE_SINGLE;
    let status = ucp_worker_create(context, &params, worker.as_mut_ptr());
    if status != UCS_OK {
        Err(Error::WorkerCreateFailed(status))
    } else {
        Ok(worker.assume_init())
    }
}

/// Exchange addresses between the two processes.
unsafe fn exchange_addrs(context: ucp_context_h, worker: ucp_worker_h, server: bool, sockaddr: SocketAddr) -> Result<Vec<u8>> {
    // Get the address of the worker
    let mut address = MaybeUninit::<*mut ucp_address_t>::uninit();
    let mut addrlen = MaybeUninit::<usize>::uninit();
    let status = ucp_worker_get_address(worker, address.as_mut_ptr(),
                                        addrlen.as_mut_ptr());
    if status != UCS_OK {
        return Err(Error::WorkerAddressFailure(status));
    }
    let address = address.assume_init();
    let addrlen = addrlen.assume_init();
    // Address of the other process
    info!("Starting address exchange");
    let other_addr = get_other_addr(server, sockaddr, address, addrlen)?;
    info!("Address exchange complete");
    ucp_worker_release_address(worker, address);
    Ok(other_addr)
}

/// Do the actual exchange and return the address of the other process.
unsafe fn get_other_addr(server: bool, sockaddr: SocketAddr, address: *const ucp_address_t, addrlen: usize) -> Result<Vec<u8>> {
    // TODO: Use bincode here
    let saddr = std::slice::from_raw_parts(address as *const u8, addrlen);
    // TODO: There has to be a better way to do this, instead of using two
    //       connections in a row.
    if server {
        let listener = TcpListener::bind(sockaddr)
            .expect("Failed to bind TCP listener");
        // First connection
        let (mut stream, _) = listener.accept()
            .expect("Failed to accept a client connection");
        // Receive the other address and then send ours
        let addr_bytes: Vec<u8> = serde_json::from_reader(&mut stream)
            .expect("Failed to parse incoming address data");
        debug!("addr_bytes: {:?}", addr_bytes);
        // Second connection
        // Now to send the server's address
        serde_json::to_writer(&mut stream, saddr)
            .expect("Failed to send address data");
        stream.flush().expect("Failed to flush stream");
        Ok(addr_bytes)
    } else {
        // First connection
        let mut stream = TcpStream::connect(sockaddr)
            .expect("Failed to connect to server for ucp address exchange");
        // Send our address and then receive the other one
        serde_json::to_writer(&mut stream, saddr)
            .expect("Failed to send address data");
        stream.flush().expect("Failed to flush stream");
        stream.shutdown(Shutdown::Write)
            .expect("Failed to shutdown stream");
        info!("Wrote address data");
        let addr_bytes = serde_json::from_reader(&mut stream)
            .expect("Failed to parse incoming address data");
        Ok(addr_bytes)
    }
}

pub(crate) fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        CStr::from_ptr(ucs_status_string(status))
            .to_str()
            .expect("Failed to convert status string")
            .to_string()
    }
}
