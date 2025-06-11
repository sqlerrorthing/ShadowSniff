#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ops::Deref;
use miniz_oxide::deflate::compress_to_vec;
use rand_chacha::rand_core::RngCore;
use utils::path::Path;
use utils::random::ChaCha20RngExt;
use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::System::Time::FileTimeToSystemTime;

pub struct ZipEntry {
    path: String,
    data: Vec<u8>,
    modified: (u16, u16)
}

#[derive(Default)]
pub struct ZipArchive {
    entries: Vec<ZipEntry>,
    comment: Option<String>,
    password: Option<String>,
    compression: ZipCompression,
}

impl Deref for ZipEntry {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Default)]
pub enum ZipCompression {
    NONE = 0u16,
    
    #[default]
    DEFLATE = 8u16
}

pub enum IntoPath<'a, 'b>
{
    Reference(&'a Path),
    Borrowed(Path),
    StringReference(&'b str)
}

impl<'a> From<IntoPath<'a, '_>> for Path {
    fn from(value: IntoPath) -> Self {
        match value {
            IntoPath::Reference(val) => val.clone(),
            IntoPath::Borrowed(val) => val,
            IntoPath::StringReference(val) => Path::new(val)
        }
    }
}

impl<'a> From<Path> for IntoPath<'a, '_> {
    fn from(value: Path) -> Self {
        IntoPath::Borrowed(value)
    }
}

impl<'a> From<&'a Path> for IntoPath<'a, '_> {
    fn from(value: &'a Path) -> Self {
        IntoPath::Reference(value)
    }
}

impl<'b> From<&'b str> for IntoPath<'_, 'b> {
    fn from(value: &'b str) -> Self {
        IntoPath::StringReference(value)
    }
}

impl ZipCompression {
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self {
            ZipCompression::DEFLATE => compress_to_vec(data, 9),
            ZipCompression::NONE => Vec::from(data)
        }
    }
}

impl ZipArchive {
    pub fn comment<S>(&mut self, comment: S) -> &mut Self
    where
        S: AsRef<str>
    {
        self.comment = Some(comment.as_ref().to_string());
        self
    }
    
    pub fn password<S>(&mut self, password: S) -> &mut Self
    where
        S: AsRef<str>
    {
        self.password = Some(password.as_ref().to_string());
        self
    }
    
    pub fn compression(&mut self, compression: ZipCompression) -> &mut Self {
        self.compression = compression;
        self
    }

    pub fn add_folder_content<'a, 'b, P>(&mut self, root: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
    {
        let root = &Path::from(root.into());
        let _ = self.add_folder_content_internal(root, root, true);
        self
    }

    pub fn add_folder<'a, 'b, P>(&mut self, folder: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
    {
        let folder = &Path::from(folder.into());
        let _ = self.add_folder_content_internal(folder, folder, false);
        self
    }

    pub fn add_file<'a, 'b, P>(&mut self, file: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
    {
        let file = &Path::from(file.into());
        let _ = self.add_file_internal(file);
        self
    }
    
    fn add_file_internal(&mut self, file: &Path) -> Option<()> {
        if !file.is_file() {
            return None
        }
        
        let full_name = file.fullname()?;
        let file_time = file.get_filetime()?;
        let data = file.read_file().ok()?;
        
        let entry = ZipEntry {
            path: full_name.to_string(),
            data,
            modified: filetime_to_dos_date_time(&file_time)
        };
        
        self.entries.push(entry);
        
        Some(())
    }

    fn add_folder_content_internal(&mut self, root: &Path, file: &Path, use_parent: bool) -> Option<()> {
        if !file.is_exists() || !root.is_exists() {
            return None
        }

        for file in file.list_files()? {
            if file.is_dir() {
                self.add_folder_content_internal(root, &file, use_parent)?
            } else if file.is_file() {
                let data = file.read_file().ok()?;
                let file_time = file.get_filetime()?;

                let rel_path = if use_parent {
                    file.strip_prefix(root.deref())?
                        .strip_prefix("\\")?
                } else {
                    file.deref()
                };

                let entry = ZipEntry {
                    path: rel_path.to_string(),
                    data,
                    modified: filetime_to_dos_date_time(&file_time)
                };

                self.entries.push(entry);
            }
        }

        Some(())
    }

    pub fn create(&self) -> Vec<u8> {
        let mut zip_data = Vec::new();
        let mut central_directory = Vec::new();
        let mut offset = 0;

        for entry in &self.entries {
            let (compression_method, compressed) = (
                    self.compression as u16,
                    self.compression.compress(&entry.data)
                );

            let crc = crc32(&entry.data);
            let path_bytes = entry.path.as_bytes();

            let(compressed, encryption_header, general_flag) =
                protect_data(crc, compressed, self.password.as_ref());

            let compressed_size = encryption_header
                .map_or(0, |h| h.len()) + compressed.len();

            let local_header = create_local_header(
                crc,
                general_flag,
                compression_method,
                entry.modified,
                compressed_size,
                entry.data.len(),
                path_bytes
            );

            zip_data.extend(&local_header);

            if let Some(header) = encryption_header.as_ref() {
                zip_data.extend(header)
            }

            zip_data.extend(&compressed);

            let central_header = create_central_header(
                crc,
                general_flag,
                compression_method,
                entry.modified,
                compressed_size,
                entry.data.len(),
                path_bytes,
                offset
            );

            central_directory.extend(&central_header);
            offset += local_header.len() + compressed_size;
        }

        zip_data.extend(&central_directory);

        let eocd = create_end_of_central_directory(
            self.entries.len(),
            central_directory.len(),
            offset,
            self.comment.as_ref()
        );

        zip_data.extend(eocd);

        zip_data
    }
}

fn filetime_to_dos_date_time(file_time: &FILETIME) -> (u16, u16) {
    let mut sys_time: SYSTEMTIME = unsafe { zeroed() };

    unsafe {
        if FileTimeToSystemTime(file_time, &mut sys_time) == 0 {
            return (0, 0);
        }
    }

    let dos_time: u16 = (sys_time.wHour << 11)
        | (sys_time.wMinute << 5) | (sys_time.wSecond / 2);

    let year = sys_time.wYear as i32;
    let dos_date: u16 = (((year - 1980) as u16) << 9)
        | sys_time.wMonth << 5
        | sys_time.wDay;

    (dos_time, dos_date)
}

fn protect_data(
    crc: u32,
    mut payload: Vec<u8>,
    password: Option<&String>
) -> (Vec<u8>, Option<[u8; 12]>, u16) {
    if let Some(password) = password {
        let (mut k0, mut k1, mut k2) = init_keys(password);
        let header = gen_encryption_header(crc, &mut k0, &mut k1, &mut k2);

        for byte in &mut payload {
            let plain = *byte;
            *byte ^= decrypt_byte(k2);
            update_keys(plain, &mut k0, &mut k1, &mut k2);
        }

        (payload, Some(header), 0x01)
    } else {
        (payload, None, 0x00)
    }
}

macro_rules! extend {
    ($($data:expr),+ $(,)?) => {{
        let mut extended = Vec::new();

        $(
            extended.extend($data);
        )+

        extended
    }};
}

fn create_local_header(
    crc: u32,
    general_flag: u16,
    compression_method: u16,
    modified: (u16, u16),
    compressed_len: usize,
    data_len: usize,
    path: &[u8]
) -> Vec<u8> {
    extend!(
        [0x50, 0x4B, 0x03, 0x04],
        20u16.to_le_bytes(),
        general_flag.to_le_bytes(),
        compression_method.to_le_bytes(),
        modified.0.to_le_bytes(),
        modified.1.to_le_bytes(),
        crc.to_le_bytes(),
        (compressed_len as u32).to_le_bytes(),
        (data_len as u32).to_le_bytes(),
        (path.len() as u16).to_le_bytes(),
        0u16.to_le_bytes(),
        path,
    )
}

#[allow(clippy::too_many_arguments)]
fn create_central_header(
    crc: u32,
    general_flag: u16,
    compression_method: u16,
    modified: (u16, u16),
    compressed_len: usize,
    data_len: usize,
    path: &[u8],
    offset: usize
) -> Vec<u8> {
    extend!(
        [0x50, 0x4B, 0x01, 0x02],
        [0x14, 0x00],
        &20u16.to_le_bytes(),
        general_flag.to_le_bytes(),
        compression_method.to_le_bytes(),
        modified.0.to_le_bytes(),
        modified.1.to_le_bytes(),
        &crc.to_le_bytes(),
        &(compressed_len as u32).to_le_bytes(),
        &(data_len as u32).to_le_bytes(),
        &(path.len() as u16).to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        &[0, 0, 0, 0],
        &(offset as u32).to_le_bytes(),
        path
    )
}

fn create_end_of_central_directory(
    entries_len: usize,
    central_size: usize,
    central_offset: usize,
    comment: Option<&String>
) -> Vec<u8> {
    let mut vec = extend!(
        &[0x50, 0x4B, 0x05, 0x06],
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        &(entries_len as u16).to_le_bytes(),
        &(entries_len as u16).to_le_bytes(),
        &(central_size as u32).to_le_bytes(),
        &(central_offset as u32).to_le_bytes()
    );

    if let Some(comment) = comment {
        let comment = comment.as_bytes();
        vec.extend(&(comment.len() as u16).to_le_bytes());
        vec.extend(comment);
    }

    vec
}

fn crc32(data: &[u8]) -> u32 {
    let polynomial: u32 = 0xEDB88320;
    let mut crc: u32 = 0xFFFFFFFF;

    for &byte in data {
        let current_byte = byte as u32;
        crc ^= current_byte;
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { polynomial } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }

    !crc
}

fn init_keys<S>(password: &S) -> (u32, u32, u32)
where 
    S: AsRef<str> + ?Sized
{
    let mut k0 = 0x12345678;
    let mut k1 = 0x23456789;
    let mut k2 = 0x34567890;
    
    for b in password.as_ref().bytes() {
        update_keys(b, &mut k0, &mut k1, &mut k2);
    }

    (k0, k1, k2)
}

fn update_keys(byte: u8, k0: &mut u32, k1: &mut u32, k2: &mut u32) {
    *k0 = crc32_byte(*k0, byte);
    *k1 = k1.wrapping_add(*k0 & 0xFF);
    *k1 = k1.wrapping_mul(134775813).wrapping_add(1);
    *k2 = crc32_byte(*k2, (*k1 >> 24) as u8);
}

fn crc32_byte(crc: u32, b: u8) -> u32 {
    let mut c = crc ^ (b as u32);
    for _ in 0..8 {
        c = if c & 1 != 0 {
            0xEDB88320 ^ (c >> 1)
        } else {
            c >> 1
        };
    }

    c
}

fn decrypt_byte(k2: u32) -> u8 {
    let temp = (k2 | 2).wrapping_mul(k2 ^ 1) >> 8;
    (temp & 0xFF) as u8
}

fn gen_encryption_header(crc: u32, k0: &mut u32, k1: &mut u32, k2: &mut u32) -> [u8; 12] {
    let mut header = [0u8; 12];

    for (idx, val) in header.iter_mut().enumerate().take(11) {
        let plain = idx as u8;
        *val = plain ^ decrypt_byte(*k2);
        update_keys(plain, k0, k1, k2);
    }

    let final_byte = (crc >> 24) as u8 ^ decrypt_byte(*k2);

    header[11] = final_byte;
    update_keys(final_byte, k0, k1, k2);

    header
}

#[cfg(test)]
mod tests {
    use crate::{decrypt_byte, gen_encryption_header, init_keys, update_keys};
    use alloc::vec::Vec;

    #[test]
    fn test_enc_dec() {
        let password = "12345";
        let data = b"hello world";

        let (mut k0, mut k1, mut k2) = init_keys(password);
        let _ = gen_encryption_header(0, &mut k0, &mut k1, &mut k2);
        let mut encrypted = Vec::new();
        for &b in data {
            let k = decrypt_byte(k2);
            encrypted.push(b ^ k);
            update_keys(b, &mut k0, &mut k1, &mut k2);
        }

        let (mut dk0, mut dk1, mut dk2) = init_keys(password);
        let _ = gen_encryption_header(0, &mut dk0, &mut dk1, &mut dk2);
        let mut decrypted = Vec::new();
        for &b in &encrypted {
            let k = decrypt_byte(dk2);
            decrypted.push(b ^ k);
            update_keys(decrypted.last().copied().unwrap(), &mut dk0, &mut dk1, &mut dk2);
        }

        assert_eq!(decrypted, data);
    }
}