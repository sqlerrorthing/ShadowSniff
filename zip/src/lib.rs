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
    comment: Option<String>
}

impl Deref for ZipEntry {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}


impl ZipArchive {
    pub fn with_comment<S>(comment: S) -> Self
    where 
        S: AsRef<str>,
    {
        Self {
            comment: Some(comment.as_ref().to_string()),
            ..Self::default()
        }
    }

    pub fn add<P>(&mut self, root: &P) -> Option<()>
    where
        P: AsRef<Path>
    {
        self.add_internal(root, root)
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

    pub fn create_zip(&self) -> Vec<u8> {
        let mut zip_data = Vec::new();
        let mut central_directory = Vec::new();
        let mut offset = 0;

        for entry in &self.entries {
            let compressed = compress_to_vec(&entry.data, 9);

            let path_bytes = entry.path.as_bytes();
            let crc = crc32(&entry.data);

            let local_header = {
                let mut h = Vec::new();

                h.extend(&[0x50, 0x4B, 0x03, 0x04]);
                h.extend(&[20, 0]);
                h.extend(&[0, 0]);
                h.extend(&[8, 0]);
                h.extend(&[0, 0, 0, 0]);
                h.extend(&crc.to_le_bytes());
                h.extend(&(compressed.len() as u32).to_le_bytes());
                h.extend(&(entry.data.len() as u32).to_le_bytes());
                h.extend(&(path_bytes.len() as u16).to_le_bytes());
                h.extend(&[0, 0]);
                h.extend(path_bytes);

                h
            };

            zip_data.extend(&local_header);
            zip_data.extend(&compressed);

            let central_header = {
                let mut c = Vec::new();

                c.extend(&[0x50, 0x4B, 0x01, 0x02]);
                c.extend(&[0x14, 0x00]);
                c.extend(&[20, 0]);
                c.extend(&[0, 0]);
                c.extend(&[8, 0]);
                c.extend(&[0, 0, 0, 0]);
                c.extend(&crc.to_le_bytes());
                c.extend(&(compressed.len() as u32).to_le_bytes());
                c.extend(&(entry.data.len() as u32).to_le_bytes());
                c.extend(&(path_bytes.len() as u16).to_le_bytes());
                c.extend(&[0, 0]);
                c.extend(&[0, 0]);
                c.extend(&[0, 0]);
                c.extend(&[0, 0]);
                c.extend(&[0, 0, 0, 0]);
                c.extend(&(offset as u32).to_le_bytes());
                c.extend(path_bytes);

                c
            };

            central_directory.extend(&central_header);
            offset += local_header.len() + compressed.len();
        }

        let central_offset = offset;
        let central_size = central_directory.len();

        zip_data.extend(&central_directory);

        let end_of_central_directory = {
            let mut e: Vec<u8> = Vec::new();

            e.extend(&[0x50, 0x4B, 0x05, 0x06]);
            e.extend(&[0, 0]);
            e.extend(&[0, 0]);
            e.extend(&(self.entries.len() as u16).to_le_bytes());
            e.extend(&(self.entries.len() as u16).to_le_bytes());
            e.extend(&(central_size as u32).to_le_bytes());
            e.extend(&(central_offset as u32).to_le_bytes());

            if let Some(comment) = &self.comment {
                let comment = comment.as_bytes();
                e.extend(&(comment.len() as u16).to_le_bytes());
                e.extend(comment);
            }

            e
        };

        zip_data.extend(end_of_central_directory);

        zip_data
    }
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