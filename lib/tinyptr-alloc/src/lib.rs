#![no_std]

use tinyptr::ptr::MutPtr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ListNode<const BASE: usize> {
    pub next: MutPtr<Self, BASE>,
    pub size: u16
}

