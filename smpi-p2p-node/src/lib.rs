//! Intra-node point-to-point provider implementation.
//!
//! Provides point-to-point communication for processes all local to the same
//! node.
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::cell::Cell;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time;
use smpi_runtime::Runtime;
use smpi_base::{Result, Error, BufRead, BufWrite, Reachability, P2PProvider};

mod queue;
use queue::Queue;

const PACKET_SIZE: usize = 1024;

/// Packet of data sent between processes.
#[repr(C)]
struct Packet {
    /// Unique message ID
    msg_id: u64,

    /// Unique type ID
    type_id: u64,

    /// Packet ID (or index) for multiple packets from the same message
    packet_id: u32,

    /// Number of bytes used in data
    len: usize,

    /// Sent data
    data: [u8; PACKET_SIZE],
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

    /// List of processes local to the node
    local_processes: Vec<u64>,

    /// Next message ID to pass
    next_msg_id: Cell<u64>,

    /// Transmitter for the progress thread
    out_packets: mpsc::Sender<(u64, Packet)>,

    /// Incoming packets stored from the progress thread
    in_packets: Arc<Mutex<Vec<(u64, Packet)>>>,
}

impl NodeP2P {
    /// Initialize the intra-node provider.
    pub fn new(runtime: Arc<Mutex<Runtime>>) -> NodeP2P {
        // Get information about this node
        let runtime2 = Arc::clone(&runtime);
        let runtime_handle = runtime2.lock().unwrap();
        let id = runtime_handle.id();
        let node_id = runtime_handle.node_id();
        let local_processes: Vec<u64> = runtime_handle.node_process_ids(node_id).collect();

        // Spawn the progress thread
        let (out_packets, mut out_packets_progress) = mpsc::channel(64);
        let in_packets_progress = Arc::new(Mutex::new(vec![]));
        let in_packets = Arc::clone(&in_packets_progress);
        let queues: HashMap<u64, Queue> = local_processes
            .iter()
            .map(|other_id| (*other_id, Queue::new(node_id, id, *other_id)))
            .collect();
        tokio::spawn(async move {
            while let Some((target, pkt)) = out_packets_progress.recv().await {
                // TODO
                // Use unix socket for now, but should use message queue or something else later on
            }
        });

        NodeP2P {
            runtime,
            local_processes,
            next_msg_id: Cell::new(0),
            out_packets,
            in_packets,
        }
    }
}

impl P2PProvider for NodeP2P {
    /// Return reachability for the process with the given ID.
    fn reachability(&self, id: u64) -> Reachability {
        if let Some(_) = self.local_processes.iter().find(|&&other_id| other_id == id) {
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
        type_id: u64,
        target: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        let msg_id = self.next_msg_id.get();
        self.next_msg_id.set(msg_id + 1);

        let out_packets = self.out_packets.clone();
        Box::pin(async move {
            // Iterate over packets and send to progress thread
            for (off, mut pkt) in split_into_packets(msg_id, type_id, size) {
                // Copy the buffer into the packet
                std::ptr::copy(buf.offset(off), &mut pkt.data as *mut _, pkt.len);
                // Send it to progress thread
                out_packets
                    .send((target, pkt))
                    .await
                    .expect("failed to send packet to progress thread");
            }

            Ok(())
        })
    }

    /// Perform a non-blocking receive.
    unsafe fn recv_nb(
        &self,
        buf: *mut u8,
        size: usize,
        type_id: u64,
        source: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        let msg_id = self.next_msg_id.get();
        self.next_msg_id.set(msg_id + 1);
        let in_packets = Arc::clone(&self.in_packets);

        Box::pin(async move {
            for pkt_id in 0..total_packets(size) {
                let pkt_id: u32 = pkt_id.try_into().unwrap();
                let packet = find_next_packet(&in_packets, type_id, source, pkt_id)
                    .await
                    .ok_or(Error::MessageTransmissionFailure);
            }

            Ok(())
        })
    }
}

/// Find the next packet by ID.
async fn find_next_packet(in_packets: &Arc<Mutex<Vec<(u64, Packet)>>>, type_id: u64, source: u64, pkt_id: u32) -> Option<Packet> {
    loop {
        // Try to lock the packets buffer
        if let Ok(ref mut packets) = in_packets.try_lock() {
            let pos = packets
                .iter()
                .position(|(id, pkt)| *id == source && pkt.type_id == type_id && pkt.packet_id == pkt_id);
            if let Some(i) = pos {
                return Some(packets.swap_remove(i).1);
            }
        }
        // Sleep a little
        time::sleep(time::Duration::from_micros(10)).await;
    }
    None
}

/// Return the total number of packets required for a given size.
fn total_packets(size: usize) -> usize {
    size / PACKET_SIZE + if (size % PACKET_SIZE) != 0 { 1 } else { 0 }
}

/// Generate an iterator over empty packets with offsets for reading from or writing to.
fn split_into_packets(msg_id: u64, type_id: u64, size: usize) -> impl Iterator<Item = (isize, Packet)> {
    let total_pkts = total_packets(size);
    (0..total_pkts)
        .map(move |i| {
            let off = i * PACKET_SIZE;
            let len = if (off + PACKET_SIZE) > size { size - off } else { PACKET_SIZE };
            (off.try_into().unwrap(), i.try_into().unwrap(), len)
        })
        .map(move |(off, packet_id, len)| {
            (
                off,
                Packet {
                    msg_id,
                    type_id,
                    packet_id,
                    len,
                    data: [0; PACKET_SIZE],
                }
            )
        })
}
