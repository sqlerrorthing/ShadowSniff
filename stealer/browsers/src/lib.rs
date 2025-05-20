#![no_std]

mod chromium;

use crate::alloc::borrow::ToOwned;
extern crate alloc;

use alloc::vec;
use tasks::Task;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};
use crate::chromium::ChromiumTask;

pub struct BrowsersTask {
    inner: CompositeTask
}

impl BrowsersTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                ChromiumTask::new()
            )
        }
    }
}

impl_composite_task_runner!(BrowsersTask, "Browsers");