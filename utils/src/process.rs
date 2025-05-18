use alloc::string::ToString;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::null_mut;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, TRUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::ReadFile;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{CreateProcessW, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE, PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW};
use crate::path::Path;
use crate::WideString;

pub unsafe fn run_file(file: &Path) -> Result<Vec<u8>, u32> {
    run_process(&file.to_string())
}

pub unsafe fn run_process(cmd: &str) -> Result<Vec<u8>, u32> {
    let mut sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: TRUE
    };
    
    let mut read_pipe: HANDLE = null_mut();
    let mut write_pipe: HANDLE = null_mut();
    
    if CreatePipe(&mut read_pipe, &mut write_pipe, &mut sa, 0) == 0 {
        return Err(GetLastError())
    }
    
    SetHandleInformation(read_pipe, HANDLE_FLAG_INHERIT, 0);
    
    let mut si: STARTUPINFOW = zeroed();
    si.cb = size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = write_pipe;
    si.hStdError = write_pipe;
    si.hStdInput = INVALID_HANDLE_VALUE;
    
    let mut pi: PROCESS_INFORMATION = zeroed();
    let mut cmd_wide = cmd.to_wide();
    
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
        &mut pi
    );
    
    CloseHandle(write_pipe);
    
    if res == 0 {
        CloseHandle(read_pipe);
        return Err(GetLastError())
    }

    WaitForSingleObject(pi.hProcess, INFINITE);
    
    let mut output = Vec::new();
    let mut buffer = [0u8; 4096];
    
    loop {
        let mut bytes_read = 0;
        let success = ReadFile(
            read_pipe,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut bytes_read,
            null_mut()
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