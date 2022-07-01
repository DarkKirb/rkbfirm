//! Constant pointer

use core::{marker::{PhantomData, Unsize}, ops::CoerceUnsized, cmp::Ordering, fmt, hash::{Hash, Hasher}};

use crate::{base_ptr, Pointable, PointerConversionError};

/// A tiny constant pointer
pub struct ConstPtr<T: Pointable + ?Sized, const BASE: usize> {
    pub(crate) ptr: u16,
    pub(crate) meta: <T as Pointable>::PointerMetaTiny,
    pub(crate) _marker: PhantomData<*const T>,
}

impl<T: Pointable + ?Sized, const BASE: usize> ConstPtr<T, BASE> {
    /// Create a new constant pointer from raw parts
    pub const fn from_raw_parts(ptr: u16, meta: <T as Pointable>::PointerMetaTiny) -> Self {
        Self {
            ptr,
            meta,
            _marker: PhantomData,
        }
    }
    /// Creates a tiny pointer unchecked
    ///
    /// # Safety
    /// This is unsafe because the address of the pointer may change.
    pub unsafe fn new_unchecked(ptr: *const T) -> Self {
        let (addr, meta) = T::extract_parts(ptr);
        let addr = if ptr.is_null() {
            0
        } else {
            addr.wrapping_sub(BASE)
        };
        Self::from_raw_parts(addr as u16, T::tiny_unchecked(meta))
    }
    /// Tries to create a tiny pointer from a pointer
    ///
    /// # Errors
    /// Returns an error if the pointer does not fit in the address space
    pub fn new(ptr: *const T) -> Result<Self, PointerConversionError<T>> {
        let (addr, meta) = T::extract_parts(ptr);
        let addr = if ptr.is_null() {
            0
        } else {
            addr.wrapping_sub(BASE)
        };
        let addr = addr
            .try_into()
            .map_err(PointerConversionError::NotInAddressSpace)?;
        let meta = T::try_tiny(meta).map_err(PointerConversionError::CannotReduceMeta)?;
        Ok(Self::from_raw_parts(addr, meta))
    }
    /// Widens the pointer
    pub fn wide(self) -> *const T {
        let addr = if self.ptr == 0 {
            0
        } else {
            usize::from(self.ptr).wrapping_add(BASE)
        };
        T::create_ptr(base_ptr::<BASE>(), addr, T::huge(self.meta))
    }
    /// Returns `true` if the pointer is null
    pub const fn is_null(self) -> bool {
        self.ptr == 0
    }
    /// Casts to a pointer of another type
    pub const fn cast<U: Pointable<PointerMetaTiny = ()>>(self) -> ConstPtr<U, BASE>
    where
        T: Pointable<PointerMetaTiny = ()>,
    {
        ConstPtr::from_raw_parts(self.ptr, self.meta)
    }
    /// Use the pointer value in a new pointer of another type
    pub const fn with_metadata_of<U: Pointable + ?Sized>(
        self,
        val: ConstPtr<U, BASE>,
    ) -> ConstPtr<U, BASE> {
        ConstPtr::from_raw_parts(self.ptr, val.meta)
    }
    // TODO: as_mut
    /// Gets the address portion of the pointer
    pub const fn addr(self) -> u16
    where
        T: Sized,
    {
        self.ptr
    }
    /// Gets the address portion of the pointer and exposeses the provenenance part
    pub const fn expose_addr(self) -> u16
    where
        T: Sized,
    {
        self.ptr
    }
    /// Creates a new pointer with the given address
    pub const fn with_addr(self, addr: u16) -> Self
    where
        T: Sized,
    {
        Self::from_raw_parts(addr, self.meta)
    }
    /// Creates a new pointer by mapping self’s address to a new one
    pub fn map_addr(self, f: impl FnOnce(u16) -> u16) -> Self
    where
        T: Sized,
    {
        self.with_addr(f(self.addr()))
    }
    /// Decompose a pointer into its address and metadata
    pub fn to_raw_parts(self) -> (ConstPtr<(), BASE>, <T as Pointable>::PointerMetaTiny) {
        (ConstPtr::from_raw_parts(self.ptr, ()), self.meta)
    }
    // TODO: as_ref
    // TODO: as_ref_unchecked
    // TODO: as_uninit_ref
    /// Calculates the offset from a pointer
    pub const unsafe fn offset(self, count: i16) -> Self
    where
        T: Sized,
    {
        self.wrapping_offset(count)
    }
    /// Calculates the offset from a pointer using wrapping arithmetic
    pub const fn wrapping_offset(mut self, count: i16) -> Self
    where
        T: Sized,
    {
        self.ptr = self
            .ptr
            .wrapping_add_signed(count.wrapping_mul(core::mem::size_of::<T>() as i16));
        self
    }
    /// Calculates the distance between two pointers
    pub const unsafe fn offset_from(self, origin: Self) -> i16
    where
        T: Sized,
    {
        self.wrapping_offset_from(origin)
    }
    /// Calculates the distance between two pointers using wrapping arithmetic
    pub const fn wrapping_offset_from(self, origin: Self) -> i16
    where
        T: Sized,
    {
        (origin.ptr as i16)
            .wrapping_sub(self.ptr as i16)
            .wrapping_div(core::mem::size_of::<T>() as i16)
    }
    /// calculates the distance between two pointers where it is known that self is equal or
    /// greater than origin
    pub unsafe fn sub_ptr(self, origin: Self) -> u16
    where
        T: Sized,
    {
        u16::try_from(self.wrapping_offset_from(origin)).unwrap_unchecked()
    }
    /// Calculates the offset from a pointer
    pub const unsafe fn add(self, count: u16) -> Self
    where
        T: Sized,
    {
        self.offset(count as i16)
    }
    /// Calculates the offset from a pointer
    pub const unsafe fn sub(self, count: u16) -> Self
    where
        T: Sized,
    {
        self.offset((count as i16).wrapping_neg())
    }
    /// Calculates the offset from a pointer using wrapping arithmetic
    pub const fn wrapping_add(self, count: u16) -> Self
    where
        T: Sized,
    {
        self.wrapping_offset(count as i16)
    }
    /// Calculates the offset from a pointer using wrapping arithmetic
    pub const fn wrapping_sub(self, count: u16) -> Self
    where
        T: Sized,
    {
        self.wrapping_offset((count as i16).wrapping_neg())
    }
    /// Reads the value from self without moving it. this leaves the memory in self unchanged.
    pub unsafe fn read(self) -> T
    where
        T: Sized,
    {
        self.wide().read()
    }
    /// Performs a volatile read of the value from self without moving it. this leaves the memory in self unchanged.
    pub unsafe fn read_volatile(self) -> T
    where
        T: Sized,
    {
        self.wide().read_volatile()
    }
    /// Reads the value from self without moving it. this leaves the memory in self unchanged.
    pub unsafe fn read_unaligned(self) -> T
    where
        T: Sized,
    {
        self.wide().read_unaligned()
    }
    // TODO: copy_to
    // TODO: copy_to_nonoverlapping
    pub const fn align_offset(self, align: u16) -> u16
    where
        T: Sized,
    {
        if !align.is_power_of_two() {
            panic!("align must be a power of two");
        }
        (self.ptr.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1))
            .wrapping_sub(self.ptr)
            .wrapping_div(core::mem::size_of::<T>() as u16)
    }
}

impl<T: Pointable<PointerMetaTiny = ()>, const BASE: usize> ConstPtr<[T], BASE> {
    pub const fn len(self) -> u16 {
        self.meta
    }
    pub const fn as_ptr(self) -> ConstPtr<T, BASE> {
        ConstPtr::from_raw_parts(self.ptr, ())
    }
    // TODO: as_uninit_slice
}

impl<T: Pointable + ?Sized, const BASE: usize> PartialEq for ConstPtr<T, BASE>
{
    fn eq(&self, other: &Self) -> bool {
        (self.ptr == other.ptr) && (self.meta == other.meta)
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Eq for ConstPtr<T, BASE> {}

impl<T: Pointable + ?Sized, const BASE: usize> Ord for ConstPtr<T, BASE>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> PartialOrd for ConstPtr<T, BASE>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Pointable + ?Sized + Unsize<U>, U: Pointable, const BASE: usize> CoerceUnsized<ConstPtr<U, BASE>> for ConstPtr<T, BASE>
where <T as Pointable>::PointerMetaTiny: CoerceUnsized<<U as Pointable>::PointerMetaTiny>
{
}

impl<T: Pointable + ?Sized, const BASE: usize> Clone for ConstPtr<T, BASE> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Pointable + ?Sized, const BASE: usize> Copy for ConstPtr<T, BASE> {}

impl<T: Pointable + ?Sized, const BASE: usize> fmt::Debug for ConstPtr<T, BASE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(self, f)
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> Hash for ConstPtr<T, BASE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(BASE);
        state.write_u16(self.ptr);
        self.meta.hash(state);
    }
}

impl<T: Pointable + ?Sized, const BASE: usize> fmt::Pointer for ConstPtr<T, BASE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.wide(), f)
    }
}


