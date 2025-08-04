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

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{NonNull, null_mut};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Memory::{
    GetProcessHeap, HeapAlloc, HeapFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
    VirtualAlloc, VirtualFree,
};

pub(crate) struct WinHeapAlloc;

#[cfg(target_arch = "x86")]
const NATURAL_HEAP_ALIGN: usize = 8;
#[cfg(target_arch = "x86_64")]
const NATURAL_HEAP_ALIGN: usize = 16;

#[cfg(target_arch = "x86")]
const NATURAL_HEAP_ALIGN_WITH_HEADER: usize = 8 + 1;
#[cfg(target_arch = "x86_64")]
const NATURAL_HEAP_ALIGN_WITH_HEADER: usize = 16 + 1;

const PAGE_SIZE: usize = 4096;

unsafe impl GlobalAlloc for WinHeapAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = GetProcessHeap();
        if heap.is_null() {
            return null_mut();
        }

        let size = layout.size();
        let align = layout.align().max(size_of::<usize>());

        match align {
            0..=NATURAL_HEAP_ALIGN => HeapAlloc(heap, 0, size) as *mut u8,
            NATURAL_HEAP_ALIGN_WITH_HEADER..PAGE_SIZE => alloc_with_header(heap, size, align),
            _ => {
                VirtualAlloc(null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) as *mut u8
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heap = GetProcessHeap();
        if ptr.is_null() || heap.is_null() {
            return;
        }

        if layout.align() <= NATURAL_HEAP_ALIGN {
            HeapFree(heap, 0, ptr as _);
            return;
        }

        let align = layout.align().max(size_of::<usize>());

        match align {
            0..=NATURAL_HEAP_ALIGN => {
                HeapFree(heap, 0, ptr as _);
            }
            NATURAL_HEAP_ALIGN_WITH_HEADER..PAGE_SIZE => {
                let header_ptr = (ptr as usize - size_of::<usize>()) as *const usize;
                let raw = header_ptr.read() as *mut u8;
                HeapFree(heap, 0, raw as _);
            }
            _ => {
                VirtualFree(ptr as _, 0, MEM_RELEASE);
            }
        }
    }
}

#[inline]
unsafe fn alloc_with_header(heap: HANDLE, size: usize, align: usize) -> *mut u8 {
    let total = size
        .checked_add(align)
        .and_then(|v| v.checked_add(size_of::<usize>()))
        .unwrap_or(0);
    if total == 0 {
        return NonNull::<u8>::dangling().as_ptr();
    }

    let raw = HeapAlloc(heap, 0, total) as *mut u8;
    if raw.is_null() {
        return null_mut();
    }

    let payload = raw.add(size_of::<usize>());
    let aligned = ((payload as usize + align - 1) & !(align - 1)) as *mut u8;

    debug_assert!((aligned as usize) >= (raw as usize + size_of::<usize>()));
    debug_assert!((aligned as usize) + size <= (raw as usize) + total);

    ((aligned as *mut usize).offset(-1)).write(raw as usize);
    aligned
}
