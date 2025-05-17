use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write;
use core::ptr::null_mut;
use windows_sys::Win32::System::Console::{GetStdHandle, WriteConsoleW, STD_OUTPUT_HANDLE};

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