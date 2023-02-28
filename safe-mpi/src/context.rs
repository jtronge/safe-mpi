use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{SocketAddr, TcpStream, TcpListener, Shutdown};
use log::{debug, info};
use ucx2_sys::{
    ucp_cleanup,
    ucp_context_h,
    ucp_worker_h,
    ucp_worker_create,
    ucp_worker_get_address,
    ucp_worker_release_address,
    ucp_worker_params_t,
    ucp_ep_create,
    ucp_ep_h,
    ucp_ep_params_t,
    ucp_address_t,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCS_THREAD_MODE_SINGLE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCS_OK,
};
use crate::communicator::Communicator;
use crate::status_to_string;

pub struct Context {
    /// UCP context
    context: ucp_context_h,
    /// Socket address of other process
    sockaddr: SocketAddr,
    /// Run as the server when exchanging internal addresses
    server: bool,
}

impl Context {
    pub(crate) fn new(
        context: ucp_context_h,
        sockaddr: SocketAddr,
        server: bool,
    ) -> Context {
        Context {
            context,
            sockaddr,
            server,
        }
    }

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

            Communicator::new(self, worker, endpoint)
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
