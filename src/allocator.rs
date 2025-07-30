use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use windows_sys::Win32::System::Memory::{GetProcessHeap, HeapAlloc, HeapFree};

pub(crate) struct WinHeapAlloc;

unsafe impl GlobalAlloc for WinHeapAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = GetProcessHeap();
        if heap.is_null() {
            return null_mut();
        }

        HeapAlloc(heap, 0, layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let heap = GetProcessHeap();
        if heap.is_null() && !ptr.is_null() {
            HeapFree(heap, 0, ptr as _);
        }
    }
}
