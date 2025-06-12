#![feature(let_chains)]
#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::vec::Vec;
use core::iter::once;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;

pub mod path;
pub mod logging;
pub mod process;
pub mod base64;
pub mod browsers;
pub mod random;

#[macro_export]
macro_rules! str {
    ($buffer:expr) => {{
        alloc::string::String::from_utf8_lossy($buffer)
    }};
}

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for str {
    fn to_wide(&self) -> Vec<u16> {
        self.encode_utf16().chain(once(0)).collect()
    }
}

pub fn get_time_milliseconds() -> u64 {
    unsafe {
        GetTickCount64()
    }
}

pub fn get_time_nanoseconds() -> u128 {
    unsafe {
        let mut freq = 0i64;
        let mut counter = 0i64;
        
        if QueryPerformanceFrequency(&mut freq) == 0 {
            return GetTickCount64() as _
        }
        
        if QueryPerformanceCounter(&mut counter) == 0 {
            return GetTickCount64() as _
        }

        (counter as u128 * 1_000_000_000u128) / freq as u128
    }
}