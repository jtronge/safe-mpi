//! Intra-node point-to-point provider implementation.
//!
//! Provides point-to-point communication for processes all local to the same
//! node.
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use tokio::sync::mpsc;
use smpi_runtime::Runtime;
use smpi_base::{Result, Error, BufRead, BufWrite, Reachability, P2PProvider};

/// Packet of data sent between processes.
#[repr(C)]
struct Packet {
    /// Unique message ID
    msg_id: u64,

    /// Packet ID (or index) for multiple packets from the same message
    packet_id: u32,

    /// Number of bytes used in data
    len: usize,

    /// Sent data
    data: [u8; 1024],
}

/// Mechanism for performing IPC between processes.
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
    local_processes: HashMap<u64, Arc<Mutex<IPCMechanism>>>,

    /// Next message ID to pass
    next_msg_id: u64,

    /// Transmitter for the progress thread
    progress_tx: mpsc::Sender<Packet>,
}

impl NodeP2P {
    /// Initialize the intra-node provider.
    pub fn new(runtime: Arc<Mutex<Runtime>>) -> NodeP2P {
        // Determine a list of processes local to this node and set up IPC
        // mechanisms for them
        let runtime2 = Arc::clone(&runtime);
        let runtime_handle = runtime2.lock().unwrap();
        let id = runtime_handle.id();
        let node_id = runtime_handle.node_id();
        let local_processes: HashMap<u64, Arc<Mutex<IPCMechanism>>> = runtime_handle
            .node_process_ids(node_id)
            .map(|proc_id| (proc_id, Arc::new(Mutex::new(IPCMechanism::new(id, proc_id, node_id)))))
            .collect();

        // Spawn the progress thread
        let (progress_tx, mut progress_rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(packet) = progress_rx.recv().await {
                // TODO
            }
        });

        NodeP2P {
            runtime,
            local_processes,
            next_msg_id: 0,
            progress_tx,
        }
    }

    /// Get the IPC Mechanism for communicating with a process with the given ID.
    fn find_ipc(&self, id: u64) -> Option<Arc<Mutex<IPCMechanism>>> {
        None
    }
}

impl P2PProvider for NodeP2P {
    /// Return reachability for the process with the given ID.
    fn reachability(&self, id: u64) -> Reachability {
        if self.local_processes.contains_key(&id) {
            // Return an estimated reachability (low latency, high bandwidth
            // for intra-node communication)
            Reachability::Reachable(1, 1000)
        } else {
            Reachability::Unreachable
        }
    }

    /// Perform a non-blocking send.
    unsafe fn send_nb(
        &self,
        buf: *const u8,
        size: usize,
        target: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        let ipc = self.find_ipc(target);
        Box::pin(async {
            let ipc = ipc.ok_or(Error::Unreachable)?;

            Ok(())
        })
    }

    /// Perform a non-blocking receive.
    unsafe fn recv_nb(
        &self,
        buf: *mut u8,
        size: usize,
        source: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        let ipc = self.find_ipc(source);
        Box::pin(async {
            let ipc = ipc.ok_or(Error::Unreachable)?;

            Err(Error::Unreachable)
        })
    }
}
