#![no_std]

extern crate alloc;

#[allow(unused_imports)]
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, TRUE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{CreateThread};
use winapi::um::synchapi::WaitForMultipleObjects;
use winapi::um::winnt::HANDLE;
use utils::path::Path;

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
                Some(name) => parent.clone() / name, 
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
                handles.len() as DWORD,
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
unsafe extern "system" fn thread_proc(param: *mut c_void) -> DWORD {
    let params = Box::from_raw(param as *mut ThreadParams);

    params.task.run(&params.path);

    drop(params);
    
    0
}
