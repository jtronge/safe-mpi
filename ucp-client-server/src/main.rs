use ucx2_sys::{
    rust_ucp_init,
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_status,
    ucp_cleanup,
    ucp_worker_create,
    ucp_worker_destroy,
    ucp_ep_create,
    ucp_listener_create,
    ucp_params_t,
    ucp_worker_params_t,
    ucp_ep_params_t,
    ucp_request_param_t,
    ucp_listener_params_t,
    ucp_err_handler_t,
    ucp_context_h,
    ucp_worker_h,
    ucp_ep_h,
    ucp_listener_h,
    UCP_PARAM_FIELD_FEATURES,
    UCS_OK,
    UCP_FEATURE_AM,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCP_EP_PARAMS_FLAGS_CLIENT_SERVER,
    UCS_THREAD_MODE_SINGLE,
    UCP_ERR_HANDLING_MODE_PEER,
    ucs_sock_addr_t,
    ucs_status_t,
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
    ucp_request_param_t__bindgen_ty_1,
    ucp_request_param_t__bindgen_ty_2,

    ucp_ep_close_nbx,
    ucp_worker_progress,
    ucp_request_check_status,
    ucp_request_free,
    UCS_INPROGRESS,

    ucp_tag_send_nbx,
    ucp_dt_iov_t,

    UCP_LISTENER_PARAM_FIELD_SOCK_ADDR,
    UCP_LISTENER_PARAM_FIELD_CONN_HANDLER,
};
use std::alloc::{alloc, dealloc, Layout};
use std::env;
use std::mem::MaybeUninit;
use std::str::FromStr;
use std::net::Ipv4Addr;
use nix::sys::socket::{
    SockaddrIn,
    SockaddrStorage,
    SockaddrLike,
    sockaddr,
};
use std::os::raw::c_void;

mod defaults;
use defaults::{
    ListenerParams,
    WorkerParams,
};

#[repr(transparent)]
#[derive(Copy, Clone)]
struct Context(ucp_context_h);

impl Context {
    fn new() -> Context {
        // The docs declare this to be UB, but this seems to be how the C API works
        let mut context = MaybeUninit::<ucp_context_h>::uninit();
    
        let mut params: ucp_params_t = unsafe { MaybeUninit::zeroed().assume_init() };
        params.field_mask = UCP_PARAM_FIELD_FEATURES.into();
        params.features = UCP_FEATURE_AM.into();
    
        let status = unsafe {
            rust_ucp_init(&params, std::ptr::null(), context.as_mut_ptr())
        };
        if status != UCS_OK {
            panic!("ucp_init() failed");
        }
        let context = unsafe { context.assume_init() };
        Context(context)
    }

    fn into_raw(&self) -> ucp_context_h {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
struct Worker(ucp_worker_h);

impl Worker {
    fn new(context: Context, params: &WorkerParams) -> Worker {
        unsafe {
            let mut worker = MaybeUninit::<ucp_worker_h>::uninit();
            let status = ucp_worker_create(context.into_raw(), params.as_ref(),
                                           worker.as_mut_ptr());
            if status != UCS_OK {
                panic!("ucp_worker_create() failed");
            }
            let worker = worker.assume_init();
            Worker(worker)
        }
    }

    fn into_raw(&self) -> ucp_worker_h {
        self.0
    }
}

struct Listener(ucp_listener_h);

impl Listener {
    fn new<'a>(worker: Worker, params: &ListenerParams<'a>) -> Listener {
        unsafe {
            let mut listener = MaybeUninit::<ucp_listener_h>::uninit();
            let status = ucp_listener_create(worker.into_raw(), params.as_ref(),
                                             listener.as_mut_ptr());
            if status != UCS_OK {
                panic!("ucp_listener_create() failed");
            }
            let listener = listener.assume_init();
            // TODO: query listening port
            Listener(listener)
        }
    }
}

struct Endpoint(ucp_ep_h);

fn server(context: Context, worker: Worker, listen_addr: SockaddrIn) {
    // TODO
    let wparams = WorkerParams::default()
        .field_mask(UCP_WORKER_PARAM_FIELD_THREAD_MODE.into())
        .thread_mode(UCS_THREAD_MODE_SINGLE);
    let data_worker = Worker::new(context, &wparams);
    let lparams = ListenerParams::default()
        .field_mask((UCP_LISTENER_PARAM_FIELD_SOCK_ADDR
                     | UCP_LISTENER_PARAM_FIELD_CONN_HANDLER).into())
        .sockaddr(&listen_addr);
    let listener = Listener::new(worker, &lparams);

    // Create a listener on the first worker

/*
    loop {
        // Wait until a request comes in
        while (arg.request) {
            ucp_worker_progress(worker);
        }

        // Create and endpoint the given client and request
        let status = create_ep(data_worker, arg.request, &server_ep);
        if status != UCS_OK {
            panic!("create_ep() failed");
        }

        work(data_worker, server_ep, 1);

        // Close endpoint
        close_ep(data_worker, server_ep, UCP_EP_CLOSE_MODE_FORCE);
        arg.request = std::ptr::null_mut();
    }
*/
}

const PORT: u16 = 5678;

// Error callback
extern "C" fn err_cb(arg: *mut c_void, ep: ucp_ep_h, status: ucs_status_t) {
    println!("In error handler");
}

// Send callback
extern "C" fn send_cb(request: *mut c_void, status: ucs_status_t, user_data: *mut c_void) {
    println!("In send handler");
}

const TAG: u64 = 100;

fn client(context: Context, worker: Worker, server_addr: &str) {
    let addr = Ipv4Addr::from_str(server_addr)
        .expect("Failed to parse listen address");
    let ip = addr.octets();
/*
    let addr = unsafe {
        let mut store = MaybeUninit::<SockaddrStorage>::uninit().assume_init();
        let sa_in = store.as_sockaddr_in_mut().unwrap();
        *sa_in = SockaddrIn::new(ip[0], ip[1], ip[2], ip[3], PORT);
        store.as_sockaddr_in().unwrap().as_ptr()
    };
*/
    let sa_in = SockaddrIn::new(ip[0], ip[1], ip[2], ip[3], PORT);
    let addr = sa_in.as_ptr();

    // Create the endpoint (based on start_client())
    let field_mask = UCP_EP_PARAM_FIELD_FLAGS
                     | UCP_EP_PARAM_FIELD_SOCK_ADDR
                     | UCP_EP_PARAM_FIELD_ERR_HANDLER
                     | UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE;
    let params = ucp_ep_params_t {
        field_mask: field_mask.into(),
        address: std::ptr::null(),
        err_mode: UCP_ERR_HANDLING_MODE_PEER,
        err_handler: ucp_err_handler_t {
            cb: Some(err_cb),
            arg: std::ptr::null_mut(),
        },
        user_data: std::ptr::null_mut(),
        flags: UCP_EP_PARAMS_FLAGS_CLIENT_SERVER,
        sockaddr: ucs_sock_addr_t {
            addr: addr as *const _,
            addrlen: std::mem::size_of::<sockaddr>().try_into().unwrap(),
        },
        conn_request: std::ptr::null_mut(),
        name: std::ptr::null(),
    };

    let mut ep = MaybeUninit::<ucp_ep_h>::uninit();
    let status = unsafe { ucp_ep_create(worker.into_raw(), &params, ep.as_mut_ptr()) };
    if status != UCS_OK {
        panic!("ucp_ep_create() failed");
    }
    let ep = unsafe { ep.assume_init() };

    // TODO: Do work
    // Allocate the buffer
    let count = 128;
    let size = 128;
    let mut iov_buf = vec![];
    let layout = Layout::array::<u8>(size).expect("Could not create layout");
    println!("Allocating IOV");
    for i in 0..count {
        unsafe {
            iov_buf.push(ucp_dt_iov_t {
                buffer: alloc(layout) as *mut _,
                length: layout.size(),
            });
        }
    }

    let buf = vec![0; count];
    let param = ucp_request_param_t {
        op_attr_mask: UCP_OP_ATTR_FIELD_CALLBACK
                      | UCP_OP_ATTR_FIELD_DATATYPE
                      | UCP_OP_ATTR_FIELD_USER_DATA,
        flags: 0,
        request: std::ptr::null_mut(),
        cb: ucp_request_param_t__bindgen_ty_1 {
            send: Some(send_cb),
        },
        // Sending an IOV
        datatype: UCP_DATATYPE_IOV.into(),
        user_data: std::ptr::null_mut(),
        reply_buffer: std::ptr::null_mut(),
        memory_type: UCS_MEMORY_TYPE_HOST.into(),
        recv_info: ucp_request_param_t__bindgen_ty_2 {
            length: std::ptr::null_mut(),
        }
    };

    unsafe {
        let req = ucp_tag_send_nbx(ep, buf.as_ptr() as *const _, buf.len(),
                                   TAG, &param);
        // Wait for the request to complete
        for i in 0..1024 {
            ucp_worker_progress(worker.into_raw());
        }
        let status = ucp_request_check_status(req);
        ucp_request_free(req);
    }
        // ucp_tag_send_nbx()
        // ucp_tag_recv_nbx()

    println!("Deallocating IOV");
    for iov in iov_buf {
        unsafe {
            dealloc(iov.buffer as *mut _, layout);
        }
    }

    // Close the server endpoint
    let param = ucp_request_param_t {
        op_attr_mask: UCP_OP_ATTR_FIELD_FLAGS,
        flags: UCP_EP_CLOSE_MODE_FLUSH,
        request: std::ptr::null_mut(),
        cb: ucp_request_param_t__bindgen_ty_1 {
            send: None,
        },
        datatype: UCP_DATATYPE_CONTIG.into(),
        user_data: std::ptr::null_mut(),
        reply_buffer: std::ptr::null_mut(),
        memory_type: UCS_MEMORY_TYPE_HOST.into(),
        recv_info: ucp_request_param_t__bindgen_ty_2 {
            length: std::ptr::null_mut(),
        },
    };
    unsafe {
        let close_req = ucp_ep_close_nbx(ep, &param);
        let status = if rust_ucs_ptr_is_ptr(close_req) != 0 {
            let mut status = UCS_OK;
            loop {
                ucp_worker_progress(worker.into_raw());
                status = ucp_request_check_status(close_req);
                if status != UCS_INPROGRESS {
                    break;
                }
            }
            ucp_request_free(close_req);
            status
        } else {
            rust_ucs_ptr_status(close_req)
        };

        if status != UCS_OK {
            panic!("Failed to close endpoint");
        }
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
        ucp_worker_destroy(worker.into_raw());
        ucp_cleanup(context.into_raw());
    }
}
