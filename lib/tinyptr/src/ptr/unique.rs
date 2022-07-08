use core::{marker::{PhantomData, Unsize}, ops::CoerceUnsized, fmt};

use crate::Pointable;

use super::{NonNull, MutPtr};

/// Unique pointer
#[repr(transparent)]
pub struct Unique<T: Pointable + ?Sized, const BASE: usize> {
    pub(crate) pointer: NonNull<T, BASE>,
    _marker: PhantomData<T>
}

unsafe impl<T: Pointable + Send + ?Sized, const BASE: usize> Send for Unique<T, BASE> {}
unsafe impl<T: Pointable + Sync + ?Sized, const BASE: usize> Sync for Unique<T, BASE> {}

impl<T: Pointable<PointerMetaTiny = ()> + Sized, const BASE: usize> Unique<T, BASE> {
    pub const fn dangling() -> Self {
        Self::from(NonNull::dangling())
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Unique<T, BASE> {
    pub const unsafe fn new_unchecked(ptr: MutPtr<T, BASE>) -> Self {
        Self::from(NonNull::new_unchecked(ptr))
    }
    pub const fn new(ptr: MutPtr<T, BASE>) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(v) => Some(Self::from(v)),
            None => None
        }
    }
    pub const fn as_ptr(self) -> MutPtr<T, BASE> {
        self.pointer.as_ptr()
    }
    // TODO: as_ref
    // TODO: as_mut
    pub const fn cast<U>(self) -> Unique<U, BASE>
    where U: Pointable<PointerMetaTiny = ()> + Sized
    {
        Unique::from(self.pointer.cast())
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Clone for Unique<T, BASE> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Copy for Unique<T, BASE> {}
impl<T: Pointable + ?Sized, U: Pointable + ?Sized, const BASE: usize> CoerceUnsized<Unique<U, BASE>> for Unique<T, BASE> where T: Unsize<U>, <T as Pointable>::PointerMetaTiny: CoerceUnsized<<U as Pointable>::PointerMetaTiny> {}
impl<T: Pointable + ?Sized, const BASE: usize> fmt::Debug for Unique<T, BASE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> fmt::Pointer for Unique<T, BASE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

// TODO: From<RefMut<T>>
impl<T: Pointable + ?Sized, const BASE: usize> const From<NonNull<T, BASE>> for Unique<T, BASE> {
    fn from(pointer: NonNull<T, BASE>) -> Self {
        Unique { pointer, _marker: PhantomData }
    }
}
