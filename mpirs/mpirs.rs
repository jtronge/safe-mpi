#[derive(Copy, Clone, Debug)]
pub enum Error {
    InternalError,
}

pub Result<T> = std::result::Result<T, Error>;

/// Trait implementing simple p2p communication on top of some lower-level library.
pub trait Communicator {
    type Request;

    /// Return the number of processes in this communicator.
    fn size(&self) -> i32;

    /// Return the current rank of the process.
    fn rank(&self) -> i32;

    /// Do a non-blocking send of data to the destination with specified tag.
    unsafe fn isend(&self, data: &[&[u8]], dest: i32, tag: i32) -> Result<Self::Request>;

    /// Do a non-blocking recv of data from the source with the specified tag.
    unsafe fn irecv(&self, data: &mut [&mut [u8]], source: i32, tag: i32) -> Result<Self::Request>;
}
