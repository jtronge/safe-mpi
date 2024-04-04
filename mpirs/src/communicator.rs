#[derive(Copy, Clone, Debug)]
pub enum Error {
    InternalError,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Trait implementing simple p2p communication on top of some lower-level library.
pub trait Communicator {
    type Request;

    /// Return the number of processes in this communicator.
    fn size(&self) -> i32;

    /// Return the current rank of the process.
    fn rank(&self) -> i32;

    /// Do a non-blocking send of data to the destination with specified tag.
    unsafe fn isend<M: Message>(&self, data: M, dest: i32, tag: i32) -> Result<Self::Request>;

    /// Do a non-blocking recv of data from the source with the specified tag.
    unsafe fn irecv<M: MessageMut>(&self, data: M, source: i32, tag: i32) -> Result<Self::Request>;

    /// Wait for all requests in list to complete.
    unsafe fn waitall(&self, requests: &[Self::Request]) -> Result<Vec<Status>>;
}

pub enum Status {
    /// Request has completed.
    Complete,
}

pub trait Message {
    /// Return a pointer to the underlying data.
    fn as_ptr(&self) -> *const u8;

    /// Return the number of bytes.
    fn count(&self) -> usize;
}

pub trait MessageMut {}

impl Message for &Vec<u32> {
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    fn count(&self) -> usize {
        self.len() * std::mem::size_of::<u32>()
    }
}

impl MessageMut for &mut Vec<u32> {}
