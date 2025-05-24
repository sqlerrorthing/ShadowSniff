#[cfg(all(debug_assertions, not(test)))]
mod panic_imports {
    pub use alloc::string::String;
    pub use core::fmt::Write;
    pub use core::ptr::null_mut;
    pub use windows_sys::Win32::System::Threading::ExitProcess;
    pub use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, MB_OK};
}

#[cfg(all(not(debug_assertions), not(test)))]
use windows_sys::Win32::System::Threading::ExitProcess;

#[cfg(all(debug_assertions, not(test)))]
use panic_imports::*;

#[cfg(all(debug_assertions, not(test)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut message = String::with_capacity(512);
    let _ = write!(&mut message, "{}\0", info);

    let title = b"ShadowSniff: Panic\0";
    let msg_ptr = message.as_ptr();

    unsafe {
        MessageBoxA(
            null_mut(),
            msg_ptr as _,
            title.as_ptr() as _,
            MB_OK | MB_ICONERROR
        );

        ExitProcess(0);   
    }
}

#[cfg(all(not(debug_assertions), not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        ExitProcess(0);
    }
}