use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::iter::once;
use core::mem::zeroed;
use core::ptr::null_mut;
use filesystem::path::Path;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
    MAX_PATH, TRUE,
};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::ReadFile;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::ProcessStatus::{K32EnumProcesses, K32GetModuleBaseNameA};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, OpenProcess, QueryFullProcessImageNameW, CREATE_NO_WINDOW, PROCESS_INFORMATION,
    PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    STARTF_USESTDHANDLES, STARTUPINFOW,
};

pub unsafe fn run_file(file: &Path) -> Result<Vec<u8>, u32> {
    run_process(&file.to_string())
}

pub unsafe fn run_process(cmd: &str) -> Result<Vec<u8>, u32> {
    let mut sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: TRUE,
    };

    let mut read_pipe: HANDLE = null_mut();
    let mut write_pipe: HANDLE = null_mut();

    if CreatePipe(&mut read_pipe, &mut write_pipe, &mut sa, 0) == 0 {
        return Err(GetLastError());
    }

    SetHandleInformation(read_pipe, HANDLE_FLAG_INHERIT, 0);

    let mut si: STARTUPINFOW = zeroed();
    si.cb = size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = write_pipe;
    si.hStdError = write_pipe;
    si.hStdInput = INVALID_HANDLE_VALUE;

    let mut pi: PROCESS_INFORMATION = zeroed();
    let mut cmd_wide: Vec<u16> = cmd.encode_utf16().chain(once(0)).collect();

    let res = CreateProcessW(
        null_mut(),
        cmd_wide.as_mut_ptr(),
        null_mut(),
        null_mut(),
        1,
        CREATE_NO_WINDOW,
        null_mut(),
        null_mut(),
        &mut si,
        &mut pi,
    );

    CloseHandle(write_pipe);

    if res == 0 {
        CloseHandle(read_pipe);
        return Err(GetLastError());
    }

    let mut output = Vec::new();
    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_read = 0;
        let success = ReadFile(
            read_pipe,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut bytes_read,
            null_mut(),
        );

        if success == 0 || bytes_read == 0 {
            break;
        }

        output.extend_from_slice(&buffer[..bytes_read as usize]);
    }

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    CloseHandle(read_pipe);

    Ok(output)
}

pub struct ProcessInfo {
    pub pid: u32,
    pub name: Arc<str>,
}

pub fn get_process_list() -> Vec<ProcessInfo> {
    let mut pids = [0u32; 1024];
    let mut bytes_returned = 0u32;
    let mut result = Vec::new();

    let success = unsafe {
        K32EnumProcesses(
            pids.as_mut_ptr(),
            size_of_val(&pids) as u32,
            &mut bytes_returned,
        )
    };

    if success == 0 {
        return result;
    }

    let count = (bytes_returned as usize) / core::mem::size_of::<u32>();

    for &pid in &pids[..count] {
        if pid == 0 {
            continue;
        }

        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid) };

        if handle == null_mut() {
            continue;
        }

        let mut name_buf = [0u8; MAX_PATH as usize];

        let len = unsafe {
            K32GetModuleBaseNameA(
                handle,
                null_mut(),
                name_buf.as_mut_ptr(),
                name_buf.len() as u32,
            )
        };

        if len > 0 {
            let name = unsafe { CStr::from_ptr(name_buf.as_ptr() as *const i8) };

            if let Ok(name_str) = name.to_str() {
                result.push(ProcessInfo {
                    pid,
                    name: Arc::from(name_str),
                });
            }
        }

        unsafe { CloseHandle(handle) };
    }

    result
}

pub fn get_process_path_by_pid(pid: u32) -> Option<Path> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;

        let success = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
        CloseHandle(handle);

        if success != 0 {
            buffer.truncate(size as usize);
            let path = String::from_utf16(&buffer).ok()?;
            Some(Path::from(path))
        } else {
            None
        }
    }
}
