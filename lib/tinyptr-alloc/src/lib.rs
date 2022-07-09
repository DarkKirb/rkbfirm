#![no_std]

use tinyptr::ptr::{MutPtr, NonNull};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ListNode<const BASE: usize> {
    pub next: MutPtr<Self, BASE>,
    pub size: u16
}

impl<const BASE: usize> ListNode<BASE> {
    pub unsafe fn next(&mut self) -> Option<&mut Self> {
        if self.next.is_null() {
            None
        } else {
            Some(&mut *(self.next.wide()))
        }
    }
    pub unsafe fn link_next(&mut self, block: NonNull<Self, BASE>) {
        (*block.as_ptr().wide()).next = self.next;
        self.next = block.as_ptr();
    }
    pub unsafe fn unlink_next(&mut self) {
        if self.next.is_null() {
            return;
        }
        self.next = (*self.next.wide()).next;
    }
}
