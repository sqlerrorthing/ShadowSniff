use crate::alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use collector::Collector;
use core::fmt::{Display, Formatter};
use core::ptr::null_mut;
use indoc::indoc;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use windows_sys::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE};

pub(super) struct InstalledAppsTask;

impl<C: Collector> Task<C> for InstalledAppsTask {
    parent_name!("InstalledApps.txt");

    fn run(&self, parent: &Path, collector: &C) {
        let Some(apps) = collect_apps() else {
            return
        };

        if apps.is_empty() {
            return
        }

        let _ = apps.into_iter()
            .map(|app| app.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
            .write_to(parent);
    }
}

fn collect_apps() -> Option<Vec<App>> {
    if unsafe {
        CoInitializeEx(null_mut(), 0)
    } < 0 {
        return None
    }

    if unsafe {
        CoInitializeSecurity(
            null_mut(),
            -1,
            null_mut(),
            null_mut(),
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            null_mut(),
            EOAC_NONE as u32,
            null_mut()
        )
    } < 0 {
        unsafe { CoUninitialize(); }
        return None
    }

    let /* cascade */: *mut c_void = null_mut();

    if unsafe {
        CoCreateInstance(
            CLSID_ACLCustomMRU
        )
    } < 0 {
        unsafe { CoUninitialize(); }
        return None
    }
}

struct App {
    name: String,
    version: String,
    id: String,
    installed_date: String
}

impl Display for App {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,
            indoc! {"
                App: {} ({})
                  Version: {}
                  Install date: {}"
            },
            self.name,
            self.id,
            self.version,
            self.installed_date
        )
    }
}