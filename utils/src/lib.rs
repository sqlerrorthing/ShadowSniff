#![feature(let_chains)]
#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::string::String;
use core::ops::Deref;
use filesystem::FileSystem;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;

pub mod base64;
pub mod logging;
pub mod process;
pub mod random;

const FLAG_MAGIC_NUMBER: u32 = 0x1F1E6 /* 🇦 */ - 'A' as u32;

#[macro_export]
macro_rules! str {
    ($buffer:expr) => {{
        alloc::string::String::from_utf8_lossy($buffer)
    }};
}

pub fn get_time_milliseconds() -> u64 {
    unsafe { GetTickCount64() }
}

pub fn get_time_nanoseconds() -> u128 {
    unsafe {
        let mut freq = 0i64;
        let mut counter = 0i64;

        if QueryPerformanceFrequency(&mut freq) == 0 {
            return GetTickCount64() as _;
        }

        if QueryPerformanceCounter(&mut counter) == 0 {
            return GetTickCount64() as _;
        }

        (counter as u128 * 1_000_000_000u128) / freq as u128
    }
}

pub fn internal_code_to_flag<S>(code: &S) -> Option<String>
where
    S: AsRef<str>,
{
    let mut flag = String::new();

    for ch in code.as_ref().trim().to_uppercase().chars() {
        if let Some(c) = char::from_u32(ch as u32 + FLAG_MAGIC_NUMBER) {
            flag.push(c);
        } else {
            return None;
        }
    }

    Some(flag)
}
