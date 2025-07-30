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

        let align = layout.align().max(size_of::<usize>());
        let size = layout.size();

        let total_size = size.checked_add(align).and_then(|v| v.checked_add(size_of::<usize>())).unwrap_or(0);
        if total_size == 0 {
            return null_mut();
        }

        let raw_ptr = HeapAlloc(heap, 0, total_size) as *mut u8;
        if raw_ptr.is_null() {
            return null_mut();
        }

        let ptr_addr = raw_ptr.add(size_of::<usize>()) as usize;
        let aligned_addr = (ptr_addr + align - 1) & !(align - 1);
        let aligned_ptr = aligned_addr as *mut u8;

        let stored_ptr = (aligned_ptr as *mut usize).offset(-1);
        stored_ptr.write(raw_ptr as usize);

        aligned_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let heap = GetProcessHeap();
        if heap.is_null() {
            return;
        }

        let stored_ptr = (ptr as *mut usize).offset(-1);
        let raw_ptr = stored_ptr.read() as *mut u8;

        HeapFree(heap, 0, raw_ptr as _);
    }
}
