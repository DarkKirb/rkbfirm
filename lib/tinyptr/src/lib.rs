//! 16 bit pointer library
//!
//! This library provides small pointers with a range of 64 kiB. This is useful for memory-limited
//! microcontrollers.
//!
//! It uses a const generic parameter to set the base address of the pointer. This allows multiple
//! small memory pools to coexist.
#![feature(coerce_unsized)]
#![feature(const_trait_impl)]
#![feature(mixed_integer_ops)]
#![feature(never_type)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(strict_provenance)]
#![feature(unsize)]
#![no_std]

use core::hash::Hash;

pub mod ptr;
mod tiny_ref;
pub use tiny_ref::*;

/// Trait that defines valid destination types for a pointer.
pub trait Pointable {
    /// The pointer metadata.
    type PointerMeta;
    /// The tiny version of the pointer metadata.
    type PointerMetaTiny: Copy + Eq + Ord + Hash;
    /// Conversion error.
    type ConversionError: core::fmt::Display + core::fmt::Debug + Clone;

    /// Try reduce the pointer metadata to a tiny version.
    ///
    /// # Errors
    /// This function returns an error if the pointer metadata does not fit into a tiny version.
    fn try_tiny(meta: Self::PointerMeta) -> Result<Self::PointerMetaTiny, Self::ConversionError>;
    /// Reduce the pointer metadata to a tiny version.
    ///
    /// # Panics
    /// This function panics if it cannot convert the pointer metadata to a tiny version.
    fn tiny(meta: Self::PointerMeta) -> Self::PointerMetaTiny {
        Self::try_tiny(meta).unwrap()
    }
    /// Reduce the pointer metadata to a tiny version, without checking
    ///
    /// # Safety
    /// This function is unsafe because it does not check if the pointer metadata fits into a tiny version.
    unsafe fn tiny_unchecked(meta: Self::PointerMeta) -> Self::PointerMetaTiny {
        Self::tiny(meta)
    }
    /// Convert a tiny version of the pointer metadata to the full version.
    fn huge(meta: Self::PointerMetaTiny) -> Self::PointerMeta;

    /// Returns an address and the pointer metadata for a pointer
    fn extract_parts(ptr: *const Self) -> (usize, Self::PointerMeta);

    /// Returns a pointer to an address in a specific address space
    fn create_ptr(base_ptr: *const (), address: usize, meta: Self::PointerMeta) -> *const Self;

    /// Returns a mutable pointer to an address in a specific address space
    fn create_ptr_mut(base_ptr: *mut (), address: usize, meta: Self::PointerMeta) -> *mut Self;
}

impl<T: Sized> Pointable for T {
    type PointerMeta = ();
    type PointerMetaTiny = ();
    type ConversionError = !;

    fn try_tiny(_: ()) -> Result<(), !> {
        Ok(())
    }
    fn tiny(_: ()) -> () {}
    fn huge(_: ()) -> () {}

    fn extract_parts(ptr: *const Self) -> (usize, ()) {
        (ptr.addr(), ())
    }
    fn create_ptr(base_ptr: *const (), address: usize, _: ()) -> *const Self {
        base_ptr.with_addr(address).cast()
    }
    fn create_ptr_mut(base_ptr: *mut (), address: usize, _: ()) -> *mut Self {
        base_ptr.with_addr(address).cast()
    }
}

impl<T: Sized> Pointable for [T] {
    type PointerMeta = usize;
    type PointerMetaTiny = u16;
    type ConversionError = <u16 as TryFrom<usize>>::Error;

    fn try_tiny(meta: usize) -> Result<u16, Self::ConversionError> {
        meta.try_into()
    }
    unsafe fn tiny_unchecked(meta: usize) -> u16 {
        meta as u16
    }
    fn huge(meta: u16) -> usize {
        meta.into()
    }
    fn extract_parts(ptr: *const Self) -> (usize, usize) {
        (ptr.as_ptr().addr(), ptr.len())
    }
    fn create_ptr(base_ptr: *const (), address: usize, meta: usize) -> *const Self {
        core::ptr::from_raw_parts(base_ptr.with_addr(address), meta)
    }
    fn create_ptr_mut(base_ptr: *mut (), address: usize, meta: usize) -> *mut Self {
        core::ptr::from_raw_parts_mut(base_ptr.with_addr(address), meta)
    }
}

pub(crate) fn base_ptr<const BASE: usize>() -> *const () {
    core::ptr::from_exposed_addr(BASE)
}
pub(crate) fn base_ptr_mut<const BASE: usize>() -> *mut () {
    core::ptr::from_exposed_addr_mut(BASE)
}

#[derive(Debug, Clone)]
pub enum PointerConversionError<T: ?Sized + Pointable> {
    /// The pointer is not in 16 bit address space
    NotInAddressSpace(<u16 as TryFrom<usize>>::Error),
    /// The pointer metadata cannot be reduced in size
    CannotReduceMeta(<T as Pointable>::ConversionError),
}
