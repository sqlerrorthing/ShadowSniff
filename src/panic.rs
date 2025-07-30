#[cfg(all(debug_assertions, not(test)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use windows_sys::Win32::System::Threading::ExitProcess;

    cfg_if::cfg_if! {
        if #[cfg(all(debug_assertions, not(test)))] {
            pub use alloc::string::String;
            pub use core::fmt::Write;
            pub use core::ptr::null_mut;
            pub use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxA};

            let mut message = String::with_capacity(512);
            let _ = write!(&mut message, "{}\0", info);

            unsafe {
                MessageBoxA(
                    null_mut(),
                    message.as_ptr() as _,
                    b"ShadowSniff: Panic\0".as_ptr() as _,
                    MB_OK | MB_ICONERROR,
                );

                ExitProcess(0);
            }
        } else {
            unsafe {
                ExitProcess(0);
            }
        }
    }
}