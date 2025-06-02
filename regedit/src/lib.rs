#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::null_mut;
use core::slice;
use utils::WideString;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ, REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_QWORD, REG_SZ};

#[cfg_attr(test, derive(Debug))]
pub enum RegistryValue {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    Dword(u32),
    Qword(u64),
    None
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
            _ => None
        }
    }
}

fn string_from_utf16_null_terminated(bytes: &[u8]) -> String {
    let utf16 = unsafe {
        slice::from_raw_parts(bytes.as_ptr() as _, bytes.len() / 2)
    };

    let len = utf16.iter().position(|&c| c == 0).unwrap_or(utf16.len());
    String::from_utf16_lossy(&utf16[..len])
}

pub fn read_registry_value<S>(base: HKEY, subkey: S, value: S) -> Result<RegistryValue, u32>
where
    S: AsRef<str>
{
    let subkey = subkey.as_ref().to_wide();
    let value = value.as_ref().to_wide();

    unsafe {
        let mut hkey: HKEY = null_mut();

        let status = RegOpenKeyExW(
            base,
            subkey.as_ptr(),
            0,
            KEY_READ,
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
        data.set_len(data_len as usize);

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

#[cfg(test)]
mod tests {

    extern crate std;

    use crate::read_registry_value;
    use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;

    #[test]
    fn read_value() {
        let value = read_registry_value(HKEY_CURRENT_USER, "SOFTWARE\\Value\\Steam\\SteamPath", "SteamPath").unwrap();
        std::println!("{:?}", value);
    }
}