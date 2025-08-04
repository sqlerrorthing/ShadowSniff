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

use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write;
use core::ptr::null_mut;
use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE, WriteConsoleW};

pub struct WindowsStdOutputWriter;

impl Write for WindowsStdOutputWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let wide: Vec<u16> = s.encode_utf16().collect();

        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == null_mut() {
                return Err(fmt::Error);
            }

            let mut written = 0;
            let res = WriteConsoleW(
                handle,
                wide.as_ptr() as *const _,
                wide.len() as u32,
                &mut written,
                null_mut(),
            );

            if res == 0 {
                return Err(fmt::Error);
            }
        }
        Ok(())
    }
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::logging::WindowsStdOutputWriter,
            format_args!($($arg)*)
        );
    }};
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! log_debug {
    ($($arg:tt)*) => {{}};
}
