use core::{marker::PhantomData, ops::Deref, borrow::Borrow};

use crate::{Pointable, ptr::NonNull};

/// Constant Tiny Reference
#[repr(transparent)]
pub struct Ref<'a, T: Pointable + ?Sized, const BASE: usize> {
    pub(crate) ptr: NonNull<T, BASE>,
    pub(crate) _marker: PhantomData<&'a T>
}

impl<T: Pointable + ?Sized, const BASE: usize> Copy for Ref<'_, T, BASE> {}
impl<T: Pointable + ?Sized, const BASE: usize> Clone for Ref<'_, T, BASE> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> Deref for Ref<'_, T, BASE> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: Reference must be valid to be constructed
        unsafe {
            &*(*self).ptr.as_ptr().wide()
        }
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> Borrow<T> for Ref<'_, T, BASE> {
    fn borrow(&self) -> &T {
        &*self
    }
}
