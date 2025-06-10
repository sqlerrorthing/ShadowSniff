#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Deref;
use miniz_oxide::deflate::compress_to_vec;
use utils::path::Path;

pub struct ZipEntry {
    path: String,
    data: Vec<u8>
}

#[derive(Default)]
pub struct ZipArchive {
    entries: Vec<ZipEntry>,
    comment: Option<String>,
    compression: ZipCompression,
}

impl Deref for ZipEntry {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum ZipCompression {
    NONE = 0,
    
    #[default]
    DEFLATE = 8
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
    
    pub fn compression(&mut self, compression: ZipCompression) -> &mut Self {
        self.compression = compression;
        self
    }

    pub fn add<P>(&mut self, root: &P) -> &mut Self
    where
        P: AsRef<Path>
    {
        let _ = self.add_internal(root, root);
        self
    }

    fn add_internal<R, F>(&mut self, root: &R, file: &F) -> Option<()>
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
                self.add_internal(root, &file)?
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
            let (compression_level, compressed) = (
                    self.compression as u8,
                    self.compression.compress(&entry.data)
                );

            let path_bytes = entry.path.as_bytes();
            let crc = crc32(&entry.data);

            let local_header = create_local_header(
                crc,
                compression_level,
                compressed.len(),
                entry.data.len(),
                path_bytes
            );

            zip_data.extend(&local_header);
            zip_data.extend(&compressed);

            let central_header = create_central_header(
                crc,
                compressed.len(),
                entry.data.len(),
                path_bytes,
                offset
            );

            central_directory.extend(&central_header);
            offset += local_header.len() + compressed.len();
        }

        zip_data.extend(&central_directory);

        let end_of_central_directory = create_end_of_central_directory(
            self.entries.len(),
            central_directory.len(),
            offset,
            self.comment.as_ref()
        );

        zip_data.extend(end_of_central_directory);

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
    compression_level: u8,
    compressed_len: usize,
    data_len: usize,
    path: &[u8]
) -> Vec<u8> {
    extend!(
        &[0x50, 0x4B, 0x03, 0x04],
        &[20, 0],
        &[0, 0],
        &[compression_level, 0],
        &[0, 0, 0, 0],
        &crc.to_le_bytes(),
        &(compressed_len as u32).to_le_bytes(),
        &(data_len as u32).to_le_bytes(),
        &(path.len() as u16).to_le_bytes(),
        &[0, 0],
        path,
    )
}

fn create_central_header(
    crc: u32,
    compressed_len: usize,
    data_len: usize,
    path: &[u8],
    offset: usize
) -> Vec<u8> {
    extend!(
        &[0x50, 0x4B, 0x01, 0x02],
        &[0x14, 0x00],
        &[20, 0],
        &[0, 0],
        &[8, 0],
        &[0, 0, 0, 0],
        &crc.to_le_bytes(),
        &(compressed_len as u32).to_le_bytes(),
        &(data_len as u32).to_le_bytes(),
        &(path.len() as u16).to_le_bytes(),
        &[0, 0],
        &[0, 0],
        &[0, 0],
        &[0, 0],
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
        &[0, 0],
        &[0, 0],
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
    const TABLE: [u32; 256] = generate_crc32_table();
    let mut crc = 0xFFFF_FFFF;

    for &b in data {
        let idx = ((crc ^ (b as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ TABLE[idx];
    }

    !crc
}

const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            crc = if (crc & 1) != 0 {
                0xEDB88320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}
