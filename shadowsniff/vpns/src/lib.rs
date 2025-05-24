#![no_std]

use crate::alloc::borrow::ToOwned;
extern crate alloc;

use tasks::Task;
use alloc::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};

pub struct VpnsTask {
    inner: CompositeTask
}

impl VpnsTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!()
        }
    }
}

impl_composite_task_runner!(VpnsTask, "Vpns");