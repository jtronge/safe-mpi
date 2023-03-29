use std::marker::PhantomData;
use std::mem::MaybeUninit;
use safe_mpi::{
    Result,
    RequestStatus,
    Tag,
    Error,
    Iov,
    MutIov,
    communicator::Communicator,
};
use flat::FlatBuffer;
use crate::data_controllers::Progress;

pub struct FlatController {
    pub comm: Communicator,
}

impl FlatController {
    pub fn new(comm: Communicator) -> FlatController {
        FlatController {
            comm,
        }
    }

    /// Send data from the buffer.
    pub fn send<T: ?Sized>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        unsafe {
            let type_id = <T as FlatBuffer>::type_id();
            let type_id_ptr = (&type_id as *const u64) as *const u8;
            let count = data.count();
            let count_ptr = (&count as *const usize) as *const u8;
            let iovecs = vec![
                Iov(type_id_ptr, std::mem::size_of::<u64>()),
                Iov(count_ptr, std::mem::size_of::<usize>()),
                Iov(data.ptr(), data.size()),
            ];
            self.comm.send(&iovecs[..], tag)
        }
    }

    /// Receive data into the buffer.
    pub fn recv<T: ?Sized>(&self, data: &mut T, tag: Tag) -> Result<()>
    where
        T: FlatBuffer,
    {
        unsafe {
            let mut type_id = MaybeUninit::<u64>::uninit();
            let mut count = MaybeUninit::<usize>::uninit();
            let iovecs = vec![
                MutIov(type_id.as_mut_ptr() as *mut _, std::mem::size_of::<u64>()),
                MutIov(count.as_mut_ptr() as *mut _, std::mem::size_of::<usize>()),
                MutIov(data.ptr_mut(), data.size()),
            ];
            self.comm.recv_iov(&iovecs[..], tag)?;
            let type_id = type_id.assume_init();
            let count = count.assume_init();
            if type_id != <T as FlatBuffer>::type_id() {
                Err(Error::MessageTypeMismatch)
            } else if count != data.count() {
                Err(Error::MessageCountMismatch)
            } else {
                Ok(())
            }
        }
    }

    /// Scope for non blocking requests.
    pub fn scope<'env, F, R>(&self, f: F) -> R
    where
        F: for<'scope> FnOnce(&mut FlatScope<'scope, 'env>) -> R,
    {
        f(&mut FlatScope {
            scope: PhantomData,
            env: PhantomData,
        })
    }
}

pub struct FlatScope<'scope, 'env: 'scope> {
    /// Invariance is over 'scope, as in iovec.rs (borrowed from std::thread)
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'scope, 'env> FlatScope<'scope, 'env> {
    /// Do a non-blocking send, returning the request index.
    pub fn isend<T: ?Sized>(&mut self, data: &'scope T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        Ok(0)
    }

    /// Do a non-blocking receive, returning the request index.
    pub fn irecv<T: ?Sized>(&mut self, data: &'scope mut T, tag: Tag) -> Result<usize>
    where
        T: FlatBuffer,
    {
        Ok(0)
    }
}

impl<'scope, 'env> Progress for FlatScope<'scope, 'env> {
    type Request = usize;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus> {
        Ok(RequestStatus::InProgress)
    }
}
