use ucp::{
    ListenerParams,
    WorkerParams,
    EndpointParams,
    RequestParam,
    Context,
    Worker,
    Request,
    Listener,
    Endpoint,
    ConnRequest,
    ucs_status_t,
    ucp_ep_h,
    ucp_dt_iov_t,
    status_to_string,
    rust_ucp_dt_make_contig,
};
use ucp::consts::{
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
    UCP_LISTENER_PARAM_FIELD_SOCK_ADDR,
    UCP_LISTENER_PARAM_FIELD_CONN_HANDLER,
    UCS_OK,
};
use std::alloc::{alloc, dealloc, Layout};
use std::env;
use std::mem::MaybeUninit;
use std::str::FromStr;
use std::net::Ipv4Addr;
use std::os::raw::c_void;
use nix::sys::socket::SockaddrIn;
use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time};

struct ListenerContext {
    request: Option<ConnRequest>,
}

/// Wait for the request to complete and then get the status
unsafe fn request_wait<F>(complete: F, worker: Worker, request: Request<'_>) -> ucs_status_t
where
    F: Fn() -> bool,
{
    while !complete() {
        worker.progress();
    }
    let status = request.status();
    request.free();
    status
}

/// Send or receive a message and wait until completion.
unsafe fn send_recv(worker: Worker, endpoint: &Endpoint, recv: bool) -> ucs_status_t {
    let mut buf = [0; COUNT];
    let complete = Rc::new(RefCell::new(false));
    let datatype_id = rust_ucp_dt_make_contig(buf.len());
    // TODO: Something is wrong with the callback lifetimes here
    if recv {
        let param = RequestParam::default()
            .op_attr_mask(UCP_OP_ATTR_FIELD_CALLBACK
                          // IMPORTANT
                          | UCP_OP_ATTR_FIELD_USER_DATA
                          | UCP_OP_ATTR_FIELD_DATATYPE)
            .datatype(datatype_id.try_into().unwrap())
            .cb_recv(|_, _, _| {
                println!("Recv complete");
                *complete.borrow_mut() = true;
            });
        let req = endpoint
            .tag_recv_nbx(worker, &mut buf, TAG, param)
            .expect("Failed to get recv request");

        request_wait(|| *complete.borrow(), worker, req)
    } else {
        let param = RequestParam::default()
            .op_attr_mask(UCP_OP_ATTR_FIELD_CALLBACK
                          // IMPORTANT
                          | UCP_OP_ATTR_FIELD_USER_DATA
                          | UCP_OP_ATTR_FIELD_DATATYPE)
            .datatype(datatype_id.try_into().unwrap())
            .cb_send(|_, status| {
                println!("Send complete");
                *complete.borrow_mut() = true;
                if status != UCS_OK {
                    println!("Bad status in send callback: {}", status_to_string(status));
                }
            });
        let req = endpoint
            .tag_send_nbx(&buf, TAG, param)
            .expect("Failed to get send request");

        request_wait(|| *complete.borrow(), worker, req)
    }
}

fn server(context: Context, worker: Worker, listen_addr: SockaddrIn) {
    let wparams = WorkerParams::default()
        .field_mask(UCP_WORKER_PARAM_FIELD_THREAD_MODE.into())
        .thread_mode(UCS_THREAD_MODE_SINGLE);
    let data_worker = Worker::new(context, &wparams);

    let listen_ctx = Rc::new(RefCell::new(ListenerContext { request: None }));
    let lparams = ListenerParams::default()
        .conn_handler(|conn_req| {
            println!("Got connection request");
            let listen_ctx = Rc::clone(&listen_ctx);
            let _ = listen_ctx.borrow_mut().request.insert(conn_req);
        })
        .field_mask((UCP_LISTENER_PARAM_FIELD_SOCK_ADDR
                     | UCP_LISTENER_PARAM_FIELD_CONN_HANDLER).into())
        .sockaddr(&listen_addr);
    // Create a listener on the first worker
    let listener = Listener::new(worker, lparams);

    loop {
        println!("listening for next connection...");
        // Wait until we receive a connection request from the client (with
        // simultaneous requests, only the first is accepted and all others are
        // rejected).
        while listen_ctx.borrow().request.is_none() {
            unsafe {
                worker.progress();
            }
        }

        let conn_request = listen_ctx.borrow_mut().request.take().unwrap();
        // Create an endpoint to the client with the data worker
        let field_mask = UCP_EP_PARAM_FIELD_ERR_HANDLER
                         | UCP_EP_PARAM_FIELD_CONN_REQUEST;
        let closed = Rc::new(RefCell::new(false));
        let ep_params = EndpointParams::default()
            .field_mask(field_mask.into())
            .conn_request(conn_request)
            .err_handler(|_ep, status| {
                println!("status in data worker error handler: {}",
                         status_to_string(status));
                *closed.borrow_mut() = true;
            });
        let data_ep = Endpoint::new(data_worker, ep_params);
        println!("created endpoint for connection");

        unsafe {
            // First message
            let status = send_recv(data_worker, &data_ep, true);
            if status != UCS_OK {
                println!("Got bad status: {}", status_to_string(status));
            }
            // Second message in reverse direction
            let status = send_recv(data_worker, &data_ep, false);
            if status != UCS_OK {
                println!("Got bad status: {}", status_to_string(status));
            }

            // Wait for the client to close the connection
            while !*closed.borrow() {
                data_worker.progress();
            }
            // Wait for request completion
            data_ep.close(data_worker, UCP_EP_CLOSE_MODE_FORCE);
        }
    }
}

const PORT: u16 = 5678;
const TAG: u64 = 100;
const COUNT: usize = 1;

/// Create and return the client endpoint
fn create_client_endpoint<'a>(
    context: Context,
    worker: Worker,
    server_addr: &'a SockaddrIn,
) -> Endpoint<'a> {
    // Create the endpoint (based on start_client())
    let field_mask = UCP_EP_PARAM_FIELD_FLAGS
                     | UCP_EP_PARAM_FIELD_SOCK_ADDR
                     | UCP_EP_PARAM_FIELD_ERR_HANDLER
                     | UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE;
    let params = EndpointParams::default()
        .field_mask(field_mask.into())
        .err_mode(UCP_ERR_HANDLING_MODE_PEER)
        .err_handler(|_ep, status| {
            println!("status: {}", status_to_string(status));
            println!("In err handler for endpoint");
        })
        .flags(UCP_EP_PARAMS_FLAGS_CLIENT_SERVER)
        .sockaddr(server_addr);

    Endpoint::new(worker, params)
}

fn addr_to_sa_in(addr: &str, port: u16) -> SockaddrIn {
    let addr = Ipv4Addr::from_str(addr)
        .expect("Failed to parse listen address");
    let ip = addr.octets();
    SockaddrIn::new(ip[0], ip[1], ip[2], ip[3], port)
}

fn client(context: Context, worker: Worker, server_addr: &str) {
    let server_addr = addr_to_sa_in(server_addr, PORT);
    let ep = create_client_endpoint(context, worker, &server_addr);

    // Allocate the buffer

    // First send message
    let status = unsafe { send_recv(worker, &ep, false) };
    if status != UCS_OK {
        println!("status: {}", status_to_string(status));
    }
    // Second message is a receive
    let status = unsafe { send_recv(worker, &ep, true) };
    if status != UCS_OK {
        println!("status: {}", status_to_string(status));
    }

    // Close the server endpoint
    unsafe {
        ep.close(worker, UCP_EP_CLOSE_MODE_FLUSH);
    }
}

fn main() {
    let mut args = env::args();
    // Skip the bin name
    args.next();
    let server_addr = args.next();

    let listen_addr = SockaddrIn::new(0, 0, 0, 0, PORT);

    let context = Context::new();
    let wparams = WorkerParams::default()
        .field_mask(UCP_WORKER_PARAM_FIELD_THREAD_MODE.into())
        .thread_mode(UCS_THREAD_MODE_SINGLE);
    let worker = Worker::new(context, &wparams);

    if let Some(server_addr) = server_addr {
        client(context, worker, &server_addr);
    } else {
        server(context, worker, listen_addr);
    }

    unsafe {
        worker.destroy();
        context.cleanup();
    }
}
