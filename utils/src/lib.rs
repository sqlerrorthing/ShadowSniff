#![feature(let_chains)]
#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;

pub mod base64;
pub mod logging;
pub mod pc_info;
pub mod process;
pub mod random;

const FLAG_MAGIC_NUMBER: u32 = 0x1F1E6 /* 🇦 */ - 'A' as u32;

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

pub fn format_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut i = 0;
    while size >= 1024.0 && i < units.len() - 1 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.2} {}", size, units[i])
}

pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    filename
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect()
}
