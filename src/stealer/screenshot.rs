use tasks::Task;
use utils::path::Path;

pub(super) struct ScreenshotTask;

impl Task for ScreenshotTask {
    unsafe fn run(&self, _parent: &Path) {
        todo!("Take screenshot")
    }
}