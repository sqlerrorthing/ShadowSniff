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
