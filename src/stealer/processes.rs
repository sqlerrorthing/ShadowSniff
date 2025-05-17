use tasks::Task;
use utils::path::Path;

pub(super) struct ProcessesTask;

impl Task for ProcessesTask {
    unsafe fn run(&self, _parent: &Path) {
        todo!("Capture the process list")
    }
}