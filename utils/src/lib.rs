#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::iter::once;

pub mod path;
pub mod logging;

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for str {
    fn to_wide(&self) -> Vec<u16> {
        self.encode_utf16().chain(once(0)).collect()
    }
}