#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Deref;
use miniz_oxide::deflate::compress_to_vec;
use rand_chacha::rand_core::RngCore;
use rand_chacha::ChaCha20Rng;
use utils::path::Path;
use utils::random::ChaCha20RngExt;

pub struct ZipEntry {
    path: String,
    data: Vec<u8>
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

    pub fn add_folder_content<P>(&mut self, root: &P) -> &mut Self
    where
        P: AsRef<Path>
    {
        let _ = self.add_folder_content_internal(root, root);
        self
    }

    pub fn add_file<P>(&mut self, file: &P) -> &mut Self
    where
        P: AsRef<Path>
    {
        let _ = self.add_file_internal(file);
        self
    }
    
    fn add_file_internal<F>(&mut self, file: &F) -> Option<()>
    where 
        F: AsRef<Path>
    {
        let file = file.as_ref();
        
        if !file.is_file() {
            return None
        }
        
        let full_name = file.fullname()?;
        let data = file.read_file().ok()?;
        
        let entry = ZipEntry {
            path: full_name.to_string(),
            data
        };
        
        self.entries.push(entry);
        
        Some(())
    }

    fn add_folder_content_internal<R, F>(&mut self, root: &R, file: &F) -> Option<()>
    where
        R: AsRef<Path>,
        F: AsRef<Path>
    {
        let file = file.as_ref();
        let root = root.as_ref();

        if !file.is_exists() || !root.is_exists() {
            return None
        }

        for file in file.list_files()? {
            if file.is_dir() {
                self.add_folder_content_internal(root, &file)?
            } else if file.is_file() {
                let bytes = file.read_file().ok()?;
                let rel_path = file.strip_prefix(root.deref())?
                    .strip_prefix("\\")?;

                let entry = ZipEntry {
                    path: rel_path.to_string(),
                    data: bytes
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
            let (compression_method, mut compressed) = (
                    self.compression as u16,
                    self.compression.compress(&entry.data)
                );

            let path_bytes = entry.path.as_bytes();
            let crc = crc32(&entry.data);
            
            let (final_data, encryption_header, general_flag) = if let Some(password) = &self.password {
                let (mut key0, mut key1, mut key2) = init_keys(password);
                let header = gen_encryption_header(crc, &mut key0, &mut key1, &mut key2);
                
                for byte in &mut compressed {
                    let k = decrypt_byte(key2);
                    *byte ^= k;
                    update_keys(*byte, &mut key0, &mut key1, &mut key2);
                }

                (compressed, header, 0x01)
            } else {
                (compressed, vec![0u8; 0], 0x00)
            };

            let compressed_size = encryption_header.len() + final_data.len();

            let local_header = create_local_header(
                crc,
                general_flag,
                compression_method,
                compressed_size,
                entry.data.len(),
                path_bytes
            );

            zip_data.extend(&local_header);
            zip_data.extend(&encryption_header);
            zip_data.extend(&final_data);

            let central_header = create_central_header(
                crc,
                general_flag as _,
                compression_method,
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
    compressed_len: usize,
    data_len: usize,
    path: &[u8]
) -> Vec<u8> {
    extend!(
        [0x50, 0x4B, 0x03, 0x04],
        20u16.to_le_bytes(),
        general_flag.to_le_bytes(),
        compression_method.to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        crc.to_le_bytes(),
        (compressed_len as u32).to_le_bytes(),
        (data_len as u32).to_le_bytes(),
        (path.len() as u16).to_le_bytes(),
        0u16.to_le_bytes(),
        path,
    )
}

fn create_central_header(
    crc: u32,
    general_flag: u16,
    compression_method: u16,
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
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
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
    S: AsRef<str>
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

fn gen_encryption_header(crc: u32, k0: &mut u32, k1: &mut u32, k2: &mut u32) -> Vec<u8> {
    let mut rng = ChaCha20Rng::from_nano_time();
    let mut header = Vec::with_capacity(12);

    for _ in 0..11 {
        let enc = rng.next_u32() as u8 ^ decrypt_byte(*k2);
        header.push(enc);
        update_keys(enc, k0, k1, k2);
    }

    let final_byte = (crc >> 24) as u8 ^ decrypt_byte(*k2);
    header.push(final_byte);
    update_keys(final_byte, k0, k1, k2);

    header
}
