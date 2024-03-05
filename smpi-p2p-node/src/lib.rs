//! Local provider implementation.
//!
//! Provides point-to-point communication for processes all local to the same
//! node.
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use smpi_runtime::Runtime;
use smpi_base::{Result, Error, BufRead, BufWrite, Reachability, P2PProvider};

#[repr(C)]
struct Packet {
    msg_id: u64,
    packet_id: u32,
    len: usize,
    data: [u8; 1024],
}

struct IPCMechanism;

impl IPCMechanism {
    /// Set up an IPC channel between this process with ID 'id' and another
    /// process with the given ID.
    fn new(id: u64, other_id: u64, node_id: u64) -> IPCMechanism {
        // TODO
        IPCMechanism
    }
}

pub struct NodeP2P {
    /// Runtime context
    runtime: Arc<Mutex<Runtime>>,

    /// IPC handles connecting to the other processes on this node
    local_processes: HashMap<u64, IPCMechanism>,

    /// Next message ID to pass
    next_msg_id: u64,
}

impl NodeP2P {
    pub fn new(runtime: Arc<Mutex<Runtime>>) -> NodeP2P {
        // Determine a list of processes local to this node and set up IPC
        // mechanisms for them
        let runtime2 = Arc::clone(&runtime);
        let runtime_handle = runtime2.lock().unwrap();
        let id = runtime_handle.id();
        let node_id = runtime_handle.node_id();
        let local_processes: HashMap<u64, IPCMechanism> = runtime_handle
            .node_process_ids(node_id)
            .map(|proc_id| (proc_id, IPCMechanism::new(id, proc_id, node_id)))
            .collect();

        NodeP2P {
            runtime,
            local_processes,
            next_msg_id: 0,
        }
    }
}

impl P2PProvider for NodeP2P {
    fn reachability(&self, id: u64) -> Reachability {
        if self.local_processes.contains_key(&id) {
            // Return an estimated reachability (low latency, high bandwidth
            // for intra-node communication)
            Reachability::Reachable(1, 1000)
        } else {
            Reachability::Unreachable
        }
    }

    unsafe fn send_nb(
        &self,
        buf: *const u8,
        size: usize,
        target: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin(async {
            Err(Error::Unreachable)
        })
    }

    unsafe fn recv_nb(
        &self,
        buf: *mut u8,
        size: usize,
        source: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin(async {
            Err(Error::Unreachable)
        })
    }
}
