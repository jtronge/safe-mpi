//! Set of traits used for passing raw structures to a message passsing system.
//! A number of these are partially based on RSMPI's Equivalence trait.
use std::any::{Any, TypeId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub unsafe trait FlatBuffer: Any {
    /// Size of this buffer in bytes
    fn size(&self) -> usize;
    /// Pointer to the buffer
    fn ptr(&self) -> *const u8;
    /// Mutable pointer to the buffer
    fn ptr_mut(&mut self) -> *mut u8;
    /// Hashed type ID of the inner type
    fn type_id() -> u64;
    /// Number of elements in the buffer (default: 1)
    fn count(&self) -> usize {
        1
    }
}

macro_rules! impl_flat_primitive {
    ($ty:ident) => {
        unsafe impl FlatBuffer for $ty {
            #[inline]
            fn size(&self) -> usize {
                std::mem::size_of::<$ty>()
            }

            #[inline]
            fn ptr(&self) -> *const u8 {
                (self as *const $ty) as *const _
            }

            #[inline]
            fn ptr_mut(&mut self) -> *mut u8 {
                (self as *mut $ty) as *mut _
            }

            #[inline]
            fn type_id() -> u64 {
                let id = TypeId::of::<Self>();
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                hasher.finish()
            }
        }
    };
}

impl_flat_primitive!(bool);
impl_flat_primitive!(isize);
impl_flat_primitive!(i8);
impl_flat_primitive!(i16);
impl_flat_primitive!(i32);
impl_flat_primitive!(i64);
impl_flat_primitive!(usize);
impl_flat_primitive!(u8);
impl_flat_primitive!(u16);
impl_flat_primitive!(u32);
impl_flat_primitive!(u64);
impl_flat_primitive!(f32);
impl_flat_primitive!(f64);
// impl_flat_primitive!(char);

unsafe impl<T: FlatBuffer> FlatBuffer for [T] {
    #[inline]
    fn size(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }

    #[inline]
    fn ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    #[inline]
    fn ptr_mut(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }

    #[inline]
    fn type_id() -> u64 {
        <T as FlatBuffer>::type_id()
    }

    #[inline]
    fn count(&self) -> usize {
        self.len()
    }
}

unsafe impl<T: FlatBuffer, const N: usize> FlatBuffer for [T; N] {
    #[inline]
    fn size(&self) -> usize {
        N * std::mem::size_of::<T>()
    }

    #[inline]
    fn ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    #[inline]
    fn ptr_mut(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }

    #[inline]
    fn type_id() -> u64 {
        <T as FlatBuffer>::type_id()
    }

    #[inline]
    fn count(&self) -> usize {
        N
    }
}

unsafe impl<T: FlatBuffer> FlatBuffer for Vec<T> {
    #[inline]
    fn size(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }

    #[inline]
    fn ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    #[inline]
    fn ptr_mut(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }

    #[inline]
    fn type_id() -> u64 {
        <T as FlatBuffer>::type_id()
    }

    #[inline]
    fn count(&self) -> usize {
        self.len()
    }
}
