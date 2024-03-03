//! Buffer traits and implementations (partially based on the data and trait
//! system used in RSMPI).
use std::any::{Any, TypeId};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

pub trait Buffer {
    /// Return the type ID of the encdoed type.
    fn type_id(&self) -> u64;

    /// Return the size of the buffer in bytes.
    fn size(&self) -> usize;
}

/// Trait for reading into a buffer.
pub unsafe trait BufRead: Buffer {
    /// Return a buffer pointer.
    fn ptr(&self) -> *const u8;
}

/// Trait for writing into a buffer.
pub unsafe trait BufWrite: Buffer {
    /// Return the buffer pointer.
    fn ptr_mut(&mut self) -> *mut u8;
}

impl<T: Copy + 'static> Buffer for Vec<T> {
    fn type_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        hasher.finish()
    }

    fn size(&self) -> usize {
        mem::size_of::<T>() * self.len()
    }
}

unsafe impl<T: Copy + 'static> BufRead for Vec<T> {
    fn ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }
}

unsafe impl<T: Copy + 'static> BufWrite for Vec<T> {
    fn ptr_mut(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }
}
