#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;
use utils::path::Path;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Threading::{CreateThread, WaitForMultipleObjects};

#[macro_export]
macro_rules! composite_task {
    ( $( $expr:expr ),* $(,)? ) => {
        CompositeTask::new(vec![
            $(
                alloc::sync::Arc::new($expr)
            ),*
        ])
    };
}

#[macro_export]
macro_rules! parent_name {
    ($name:expr) => {
        fn parent_name(&self) -> Option<alloc::string::String> {
            Some(obfstr::obfstr!($name).to_owned())
        }
    };
}

#[macro_export]
macro_rules! impl_composite_task_runner {
    ($task_type:ty) => {
        impl Task for $task_type {
            unsafe fn run(&self, parent: &utils::path::Path) {
                unsafe {
                    self.inner.run(parent);
                }
            }
        }
    };

    ($task_type:ty, $parent_name:expr) => {
        impl Task for $task_type {
            $crate::parent_name!($parent_name);

            unsafe fn run(&self, parent: &utils::path::Path) {
                unsafe {
                    self.inner.run(parent);
                }
            }
        }
    };
}

pub trait Task: Send + Sync {
    fn parent_name(&self) -> Option<String> {
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
        match self.subtasks.len() {
            0 => return,
            1 => {
                let task = &self.subtasks[0];
                task.run(&task_path(task, parent));
            }
            _ => run_tasks(&self.subtasks, parent)
        }
    }
}

unsafe fn run_tasks(tasks: &Vec<Arc<dyn Task>>, parent: &Path) {
    let mut handles: Vec<HANDLE> = Vec::new();

    for task in tasks.clone() {
        let params = Box::new(ThreadParams {
            task: task.clone(),
            path: task_path(&task, parent)
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

    WaitForMultipleObjects(
        handles.len() as _,
        handles.as_ptr(),
        TRUE,
        0xFFFFFFFF,
    );

    for handle in handles {
        CloseHandle(handle);
    }
}

fn task_path(task: &Arc<dyn Task>, parent: &Path) -> Path {
    match task.parent_name() {
        Some(name) => parent / name,
        None => parent.clone(),
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
