//! Message passing runtime and process management code.

pub struct Runtime;

impl Runtime {
    pub fn new() -> Runtime {
        Runtime
    }

    /// Return the total number of processes.
    pub fn size(&self) -> u64 {
        1
    }

    /// Return the unique ID for this process among all processes.
    pub fn id(&self) -> u64 {
        0
    }

    /// Get the unique ID for this node.
    pub fn node_id(&self) -> u64 {
        0
    }

    /// Return an iterator over the IDs of all processes that are on the given
    /// node.
    pub fn node_process_ids(&self, node_id: u64) -> impl Iterator<Item = u64> {
        (0..1)
    }
}
