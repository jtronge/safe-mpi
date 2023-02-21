use log::{debug, error, info};
use serde::{Serialize, de::DeserializeOwned};
use ucx2_sys::{
    rust_ucp_init,
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_is_err,
    rust_ucs_ptr_status,
    rust_ucp_dt_make_contig,
    ucp_cleanup,
    ucp_worker_create,
    ucp_worker_get_address,
    ucp_worker_release_address,
    ucp_worker_progress,
    ucs_status_string,
    ucp_ep_create,
    ucp_tag_send_nbx,
    ucp_tag_recv_nbx,
    ucp_stream_send_nbx,
    ucp_stream_recv_nbx,
    ucp_context_h,
    ucp_worker_h,
    ucp_ep_h,
    ucp_params_t,
    ucp_worker_params_t,
    ucp_request_param_t,
    ucp_ep_params_t,
    ucp_address_t,
    ucp_tag_recv_info_t,
    ucs_status_t,
    UCP_PARAM_FIELD_FEATURES,
    UCP_FEATURE_TAG,
    UCP_FEATURE_STREAM,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCS_THREAD_MODE_SINGLE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCP_OP_ATTR_FIELD_DATATYPE,
    UCP_OP_ATTR_FIELD_CALLBACK,
    UCP_OP_ATTR_FIELD_USER_DATA,
    UCP_DATATYPE_CONTIG,
    UCS_OK,
    UCS_INPROGRESS,
};
use nix::sys::socket::{
    SockaddrIn,
    SockaddrLike,
};
use std::ffi::CStr;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddr, TcpStream, TcpListener, Shutdown};
use std::os::raw::c_void;
use std::result::Result as StandardResult;
use std::str::FromStr;
use std::rc::Rc;
use std::cell::Cell;
// TODO: Use bincode
use serde_json;
use bincode;

mod request;
use request::Request;

/// Default port to communicate on for now
const PORT: u16 = 5588;

pub struct Context {
    /// UCP context
    context: ucp_context_h,
    /// Socket address of other process
    sockaddr: SocketAddr,
    /// Run as the server when exchanging internal addresses
    server: bool,
}

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
            Ok(Context {
                context,
                sockaddr,
                server,
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

            // Get the address of the worker
            let mut address = MaybeUninit::<*mut ucp_address_t>::uninit();
            let mut addrlen = MaybeUninit::<usize>::uninit();
            let status = ucp_worker_get_address(worker, address.as_mut_ptr(),
                                                addrlen.as_mut_ptr());
            if status != UCS_OK {
                panic!("Failed to get the address of the worker: {}",
                       status_to_string(status));
            }
            let address = address.assume_init();
            let addrlen = addrlen.assume_init();
            // Address of the other process
            info!("Starting address exchange");
            let other_addr = self.exchange_addrs(address, addrlen);
            info!("Address exchange complete");
            ucp_worker_release_address(worker, address);

            // Now create the single endpoint (this will change for multiple processes)
            let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
            let mut params = MaybeUninit::<ucp_ep_params_t>::uninit().assume_init();
            params.field_mask = UCP_EP_PARAM_FIELD_REMOTE_ADDRESS.into();
            params.address = other_addr.as_ptr() as *const _;
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

    /// Exchange addresses between two processes.
    unsafe fn exchange_addrs(&self, address: *const ucp_address_t, addrlen: usize) -> Vec<u8> {
        // TODO: Use bincode here
        let saddr = std::slice::from_raw_parts(address as *const u8, addrlen);
        // TODO: There has to be a better way to do this, instead of using two
        //       connections in a row.
        if self.server {
            let listener = TcpListener::bind(self.sockaddr)
                .expect("Failed to bind TCP listener");
            // First connection
            let (mut stream, addr) = listener.accept()
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
            addr_bytes
        } else {
            // First connection
            let mut stream = TcpStream::connect(self.sockaddr)
                .expect("Failed to connect to server for ucp address exchange");
            // Send our address and then receive the other one
            serde_json::to_writer(&mut stream, saddr)
                .expect("Failed to send address data");
            stream.flush().expect("Failed to flush stream");
            stream.shutdown(Shutdown::Write)
                .expect("Failed to shutdown stream");
            info!("Wrote address data");
            serde_json::from_reader(&mut stream)
                .expect("Failed to parse incoming address data")
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

impl<'a> Communicator<'a> {
    pub fn send(&self, buf: &[u8]) {
        unsafe {
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            let req = ucp_tag_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                &param,
            );

            self.wait_loop(req, None);
        }
    }

    pub fn recv(&self, buf: &mut [u8]) {
        unsafe {
            let done = Rc::new(Cell::new(false));
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK
                                 | UCP_OP_ATTR_FIELD_USER_DATA;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.recv = Some(tag_recv_nbx_callback);
            let done_ptr = Rc::into_raw(Rc::clone(&done)) as *mut _;
            param.user_data = done_ptr;
            let req = ucp_tag_recv_nbx(
                self.worker,
                buf.as_mut_ptr() as *mut _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                0,
                &param,
            );

            let done_clone = Rc::clone(&done);
            self.wait_loop(req, Some(done_clone));
            // Convert the pointer back to an Rc and avoid the memory leak
            Rc::from_raw(done_ptr);
        }
    }

    pub fn isend<T>(&self, data: T) -> Request<T>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe {
            let buf: Vec<u8> = bincode::serialize(&data)
                .expect("Failed to serialize data");
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            let req = ucp_tag_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                &param,
            );

            Request::new(Some(data), Some(buf), Box::new(false), req, self.worker)
        }
    }

    pub fn irecv<T>(&self) -> Request<T>
    where
        T: Serialize + DeserializeOwned,
    {
        // TODO
        Request::new(None, None, Box::new(false), std::ptr::null_mut(), self.worker)
    }

    /// Do a streaming send.
    pub fn stream_send(&self, buf: &[u8]) {
        unsafe {
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.send = Some(send_nbx_callback);
            let req = ucp_stream_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                &param,
            );
            self.wait_loop(req, None);
        }
    }

    pub fn stream_recv(&self, buf: &mut [u8]) {
        unsafe {
            let mut length = 0;
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.recv_stream = Some(stream_recv_nbx_callback);
            let req = ucp_stream_recv_nbx(
                self.endpoint,
                buf.as_ptr() as *mut _,
                buf.len() * std::mem::size_of::<u8>(),
                &mut length,
                &param,
            );
            if req == std::ptr::null_mut() {
                // TODO: What to do with the length?
            }
            self.wait_loop(req, None);
        }
    }

    /// Wait for a request to finish.
    ///
    /// TODO: Perhaps this done variable is not following proper safety protocols
    unsafe fn wait_loop(&self, req: *const c_void, done: Option<Rc<Cell<bool>>>) {
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
                ucp_worker_progress(self.worker);
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
            if done.is_some() && done.as_ref().unwrap().get() {
                break;
            }
            i += 1;
        }
    }
}

extern "C" fn send_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    user_data: *mut c_void,
) {
    panic!("In send_nbx_callback with status: {}", status_to_string(status));
}

unsafe extern "C" fn tag_recv_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    tag_info: *const ucp_tag_recv_info_t,
    user_data: *mut c_void,
) {
    if status != UCS_OK {
        panic!("Request failed with: {}", status_to_string(status));
    }
    let done = Rc::from_raw(user_data as *const Cell<bool>);
    (*done).set(true);
    info!("Received value");
    Rc::into_raw(done);
}

extern "C" fn stream_recv_nbx_callback(
    req: *mut c_void,
    status: ucs_status_t,
    length: usize,
    user_data: *mut c_void,
) {
    panic!("In stream_recv_nbx_callback with length {} and status {}",
           length, status_to_string(status));
}

pub(crate) fn status_to_string(status: ucs_status_t) -> String {
    unsafe {
        CStr::from_ptr(ucs_status_string(status))
            .to_str()
            .expect("Failed to convert status string")
            .to_string()
    }
}
