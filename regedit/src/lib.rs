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

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::iter::once;
use core::ptr::null_mut;
use core::slice;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, KEY_READ, KEY_WOW64_64KEY, REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_QWORD, REG_SZ,
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
};

#[cfg_attr(test, derive(Debug))]
pub enum RegistryValue {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    Dword(u32),
    Qword(u64),
    None,
}

impl RegistryValue {
    fn from_raw(raw: Vec<u8>, reg_type: u32) -> Self {
        use RegistryValue::*;

        match reg_type {
            REG_SZ | REG_EXPAND_SZ => String(string_from_utf16_null_terminated(&raw)),
            REG_BINARY => Binary(raw),
            REG_DWORD => {
                if raw.len() >= 4 {
                    Dword(u32::from_be_bytes(raw[..4].try_into().unwrap()))
                } else {
                    None
                }
            }
            REG_QWORD => {
                if raw.len() >= 8 {
                    Qword(u64::from_be_bytes(raw[..8].try_into().unwrap()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn string_from_utf16_null_terminated(bytes: &[u8]) -> String {
    let utf16 = unsafe { slice::from_raw_parts(bytes.as_ptr() as _, bytes.len() / 2) };

    let len = utf16.iter().position(|&c| c == 0).unwrap_or(utf16.len());
    String::from_utf16_lossy(&utf16[..len])
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn read_registry_value<K, V>(base: HKEY, subkey: K, value: V) -> Result<RegistryValue, u32>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let subkey = to_wide(subkey.as_ref());
    let value = to_wide(value.as_ref());

    unsafe {
        let mut hkey: HKEY = null_mut();

        let status = RegOpenKeyExW(
            base,
            subkey.as_ptr(),
            0,
            KEY_READ | KEY_WOW64_64KEY,
            &mut hkey,
        );

        if status != ERROR_SUCCESS {
            return Err(status);
        }

        let mut data_len: u32 = 0;
        let mut reg_type: u32 = 0;

        let result = RegQueryValueExW(
            hkey,
            value.as_ptr(),
            null_mut(),
            &mut reg_type,
            null_mut(),
            &mut data_len,
        );

        if result != ERROR_SUCCESS {
            RegCloseKey(hkey);
            return Err(result);
        }

        let mut data = Vec::<u8>::with_capacity(data_len as usize);

        let result = RegQueryValueExW(
            hkey,
            value.as_ptr(),
            null_mut(),
            &mut reg_type,
            data.as_mut_ptr(),
            &mut data_len,
        );

        RegCloseKey(hkey);

        if result != ERROR_SUCCESS {
            return Err(result);
        }

        data.set_len(data_len as usize);
        Ok(RegistryValue::from_raw(data, reg_type))
    }
}

fn to_wide(str: &str) -> Vec<u16> {
    str.encode_utf16().chain(once(0)).collect()
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::read_registry_value;
    use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;

    #[test]
    fn read_product_name() {
        let value = read_registry_value(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "ProductName",
        );

        assert!(value.is_ok());
    }
}
