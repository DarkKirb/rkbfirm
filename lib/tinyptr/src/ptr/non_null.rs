use core::{num::NonZeroU16, marker::{PhantomData, Unsize}, ops::CoerceUnsized, fmt, cmp::Ordering, hash};

use crate::Pointable;

use super::MutPtr;

/// `*mut T` but non-zero and covariant
pub struct NonNull<T: Pointable + ?Sized, const BASE: usize> {
    pub(crate) ptr: NonZeroU16,
    pub(crate) meta: <T as Pointable>::PointerMetaTiny,
    pub(crate) _marker: PhantomData<MutPtr<T, BASE>>
}

impl<T: Pointable<PointerMetaTiny = ()> + Sized, const BASE: usize> NonNull<T, BASE> {
    /// Creates a dangling but well-aligned `NonNull`
    pub const fn dangling() -> Self {
        // SAFE: align_of is never 0
        unsafe {
            Self::new_unchecked(MutPtr::from_raw_parts(core::mem::align_of::<T>() as u16, ()))
        }
    }
    // TODO: as_uninit_ref
    // TODO: as_uninit_mut
}
impl<T: Pointable + ?Sized, const BASE: usize> NonNull<T, BASE> {
    pub const unsafe fn new_unchecked(ptr: MutPtr<T, BASE>) -> Self {
        NonNull {
            ptr: NonZeroU16::new_unchecked(ptr.ptr),
            meta: ptr.meta,
            _marker: PhantomData
        }
    }
    pub const fn new(ptr: MutPtr<T, BASE>) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            // SAFETY: Just checked for null
            unsafe {
                Some(Self::new_unchecked(ptr))
            }
        }
    }
    pub const fn from_raw_parts(
        data_address: NonNull<(), BASE>,
        metadata: <T as Pointable>::PointerMetaTiny
    ) -> Self {
        unsafe {
            Self::new_unchecked(MutPtr::from_raw_parts(data_address.as_ptr().addr(), metadata))
        }
    }
    pub const fn to_raw_parts(self) -> (NonNull<(), BASE>, <T as Pointable>::PointerMetaTiny) {
        (self.cast(), self.meta)
    }
    pub const fn addr(self) -> NonZeroU16 {
        self.ptr
    }
    pub const fn with_addr(self, addr: NonZeroU16) -> Self
    where
        T: Sized
    {
        Self {
            ptr: addr,
            meta: self.meta,
            _marker: PhantomData
        }
    }
    pub fn map_addr(self, f: impl FnOnce(NonZeroU16) -> NonZeroU16) -> Self
    where T: Sized
    {
        self.with_addr(f(self.addr()))
    }
    pub const fn as_ptr(self) -> MutPtr<T, BASE> {
        MutPtr::from_raw_parts(self.ptr.get(), self.meta)
    }
    // TODO: as_ref
    // TODO: as_mut
    pub const fn cast<U>(self) -> NonNull<U, BASE>
    where U: Pointable<PointerMetaTiny = ()>
    {
        NonNull {
            ptr: self.ptr,
            meta: (),
            _marker: PhantomData
        }
    }
}

impl<T: Pointable<PointerMetaTiny = ()>, const BASE: usize> NonNull<[T], BASE> {
    pub const fn slice_from_raw_parts(data: NonNull<T, BASE>, len: u16) -> Self {
        Self {
            ptr: data.ptr,
            meta: len,
            _marker: PhantomData
        }
    }
    pub const fn len(self) -> u16 {
        self.meta
    }
    pub const fn as_non_null_ptr(self) -> NonNull<T, BASE> {
        NonNull {
            ptr: self.ptr,
            meta: (),
            _marker: PhantomData
        }
    }
    pub const fn as_mut_ptr(self) -> MutPtr<T, BASE> {
        self.as_non_null_ptr().as_ptr()
    }
    // TODO: as_uninit_slice
    // TODO: as_uninit_slice_mut
}

impl<T: Pointable + ?Sized, const BASE: usize> Clone for NonNull<T, BASE> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Copy for NonNull<T, BASE> {}
impl<T: Pointable + ?Sized, U: Pointable + ?Sized, const BASE: usize> CoerceUnsized<NonNull<U, BASE>> for NonNull<T, BASE> where T: Unsize<U>, <T as Pointable>::PointerMetaTiny: CoerceUnsized<<U as Pointable>::PointerMetaTiny> {}

impl<T: Pointable + ?Sized, const BASE: usize> fmt::Debug for NonNull<T, BASE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> fmt::Pointer for NonNull<T, BASE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> Eq for NonNull<T, BASE> {}
impl<T: Pointable + ?Sized, const BASE: usize> PartialEq for NonNull<T, BASE> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> Ord for NonNull<T, BASE> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> PartialOrd for NonNull<T, BASE> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> hash::Hash for NonNull<T, BASE> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state)
    }
}
// TODO: From<Unique<T>>
// TODO: From<RefMut<T>>
// TODO: From<Ref<T>>
