use ucp::{
    ListenerParams,
    WorkerParams,
    EndpointParams,
    RequestParam,
    Context,
    Worker,
    Listener,
    Endpoint,
    ConnRequest,
    ucs_status_t,
    ucp_ep_h,
    ucp_dt_iov_t,
    ucs_status_string,
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

struct ListenerContext {
    request: Option<ConnRequest>,
}

fn server(context: Context, worker: Worker, listen_addr: SockaddrIn) {
    // TODO
    let wparams = WorkerParams::default()
        .field_mask(UCP_WORKER_PARAM_FIELD_THREAD_MODE.into())
        .thread_mode(UCS_THREAD_MODE_SINGLE);
    let data_worker = Worker::new(context, &wparams);
    let listen_ctx = Rc::new(RefCell::new(ListenerContext { request: None }));
    let lparams = ListenerParams::default()
        .conn_handler(|conn_req| {
            let listen_ctx = Rc::clone(&listen_ctx);
            let _ = *listen_ctx.borrow_mut().request.insert(conn_req);
        })
        .field_mask((UCP_LISTENER_PARAM_FIELD_SOCK_ADDR
                     | UCP_LISTENER_PARAM_FIELD_CONN_HANDLER).into())
        .sockaddr(&listen_addr);
    let listener = Listener::new(worker, lparams);

    // Create a listener on the first worker
}

const PORT: u16 = 5678;
const TAG: u64 = 100;

unsafe fn client_work_loop(ep: Endpoint) {
    // Send 1024 messages
    for _ in 0..1024 {
    }
}

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
        .err_handler(|_, _| {
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

    // Shared boolean indicating whether a request has completed or not
    let complete = Rc::new(RefCell::new(false));

    // TODO: Do work
    // Allocate the buffer

    let count = 128;
    let buf = vec![0; count];
    let param = RequestParam::default()
        .op_attr_mask(UCP_OP_ATTR_FIELD_CALLBACK
                      | UCP_OP_ATTR_FIELD_DATATYPE
                      | UCP_OP_ATTR_FIELD_USER_DATA)
        .cb_send(|_, status| {
            if status != UCS_OK {
                eprintln!("bad request sent");
            }
            let complete = Rc::clone(&complete);
            *complete.borrow_mut() = true;
        })
        .datatype(UCP_DATATYPE_CONTIG.into())
        .memory_type(UCS_MEMORY_TYPE_HOST.into());

    let status = unsafe {
        let req = ep.tag_send_nbx(&buf, TAG, param).expect("tag_send_nbx() failed");

        // Wait for the request to complete
        while !*complete.borrow() {
            worker.progress();
        }
        let status = req.status();
        req.free();
        status
    };
    if status != UCS_OK {
        let s = unsafe { ucs_status_string(status) };
        println!("{:?}", s);
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

    if server_addr.is_some() {
        client(context, worker, &server_addr.unwrap());
    } else {
        server(context, worker, listen_addr);
    }

    unsafe {
        worker.destroy();
        context.cleanup();
    }
}
