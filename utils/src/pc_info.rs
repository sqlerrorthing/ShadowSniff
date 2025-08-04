/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use alloc::string::String;
use alloc::sync::Arc;
use obfstr::obfstr as s;
use regedit::{RegistryValue, read_registry_value};
use windows_sys::Win32::Foundation::MAX_PATH;
use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;
use windows_sys::Win32::System::WindowsProgramming::{GetComputerNameW, GetUserNameW};

#[derive(Clone)]
pub struct PcInfo {
    pub computer_name: Arc<str>,
    pub user_name: Arc<str>,
    pub product_name: Arc<str>,
}

impl PcInfo {
    pub fn retrieve() -> Self {
        Self {
            computer_name: get_computer_name().unwrap_or(Arc::from("Unknown")),
            user_name: get_user_name().unwrap_or(Arc::from("Unknown")),
            product_name: get_product_name().unwrap_or(Arc::from("Unknown")),
        }
    }
}

fn get_product_name() -> Option<Arc<str>> {
    let RegistryValue::String(name) = read_registry_value(
        HKEY_LOCAL_MACHINE,
        s!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
        s!("ProductName"),
    )
    .ok()?
    else {
        return None;
    };

    Some(name.into())
}

fn get_computer_name() -> Option<Arc<str>> {
    let mut buffer = [0u16; MAX_PATH as usize + 1];
    let mut size = buffer.len() as u32;
    let success = unsafe { GetComputerNameW(buffer.as_mut_ptr(), &mut size) };
    if success != 0 {
        let slice = &buffer[..size as usize];
        Some(Arc::from(String::from_utf16(slice).ok()?.into_boxed_str()))
    } else {
        None
    }
}

fn get_user_name() -> Option<Arc<str>> {
    let mut buffer = [0u16; MAX_PATH as usize + 1];
    let mut size = buffer.len() as u32;
    let success = unsafe { GetUserNameW(buffer.as_mut_ptr(), &mut size) };
    if success != 0 && size > 0 {
        let slice = &buffer[..(size - 1) as usize];
        Some(Arc::from(String::from_utf16(slice).ok()?.into_boxed_str()))
    } else {
        None
    }
}
