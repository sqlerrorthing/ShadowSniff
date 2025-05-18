#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;
use utils::path::Path;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Threading::{CreateThread, WaitForMultipleObjects};

pub trait Task: Send + Sync {
    fn parent_name(&self) -> Option<&'static str> {
        None
    }
    
    unsafe fn run(&self, parent: &Path);
}

pub struct CompositeTask {
    subtasks: Vec<Arc<dyn Task>>
}

impl CompositeTask {
    pub fn new(subtasks: Vec<Arc<dyn Task>>) -> CompositeTask {
        Self {
            subtasks
        }
    }
}

impl Task for CompositeTask {
    unsafe fn run(&self, parent: &Path) {
        let mut handles: Vec<HANDLE> = Vec::new();
        
        for task in self.subtasks.clone() {
            let path = match task.parent_name() {
                Some(name) => parent / name,
                None => parent.clone(),
            };
            
            let params = Box::new(ThreadParams {
                task: task.clone(),
                path
            });
            
            let handle = unsafe {
                CreateThread(
                    null_mut(),
                    0,
                    Some(thread_proc),
                    Box::into_raw(params) as *mut _,
                    0,
                    null_mut()
                )
            };
            
            if !handle.is_null() {
                handles.push(handle);
            }
        }

        unsafe {
            WaitForMultipleObjects(
                handles.len() as _,
                handles.as_ptr(),
                TRUE,
                0xFFFFFFFF,
            );
        }

        for handle in handles {
            unsafe { CloseHandle(handle) };
        }
    }
}

struct ThreadParams {
    task: Arc<dyn Task>,
    path: Path,
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn thread_proc(param: *mut c_void) -> u32 {
    let params = Box::from_raw(param as *mut ThreadParams);

    params.task.run(&params.path);

    drop(params);
    
    0
}
