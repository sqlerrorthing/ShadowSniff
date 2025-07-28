#![no_std]

extern crate alloc;
mod create;

use crate::create::create_zip;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ops::Deref;
use filesystem::path::Path;
use filesystem::{FileSystem, FileSystemExt};
use miniz_oxide::deflate::compress_to_vec;
use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::System::Time::FileTimeToSystemTime;

pub struct ZipEntry {
    path: String,
    data: Vec<u8>,
    modified: (u16, u16),
}

#[derive(Default)]
pub struct ZipArchive {
    entries: Vec<ZipEntry>,
    comment: Option<Arc<str>>,
    password: Option<Arc<str>>,
    compression: ZipCompression,
}

impl AsRef<ZipArchive> for ZipArchive {
    fn as_ref(&self) -> &ZipArchive {
        self
    }
}

impl Deref for ZipEntry {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

#[derive(Copy, Clone)]
pub enum ZipCompression {
    NONE,

    DEFLATE(u8),
}

impl Default for ZipCompression {
    fn default() -> Self {
        ZipCompression::DEFLATE(10)
    }
}

impl ZipCompression {
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self {
            ZipCompression::DEFLATE(level) => compress_to_vec(data, *level),
            ZipCompression::NONE => Vec::from(data),
        }
    }

    pub fn method(&self) -> u16 {
        match self {
            ZipCompression::DEFLATE(_) => 8u16,
            ZipCompression::NONE => 0u16,
        }
    }
}

impl ZipArchive {
    pub fn comment<S>(mut self, comment: S) -> Self
    where
        S: AsRef<str>,
    {
        self.comment = Some(Arc::from(comment.as_ref()));
        self
    }

    pub fn password<S>(mut self, password: S) -> Self
    where
        S: AsRef<str>,
    {
        assert!(password.as_ref().is_ascii(), "Password must be ASCII only");
        self.password = Some(Arc::from(password.as_ref()));
        self
    }

    pub fn compression(mut self, compression: ZipCompression) -> Self {
        self.compression = compression;
        self
    }

    pub fn add_folder_content<F, P>(mut self, filesystem: &F, root: P) -> Self
    where
        P: AsRef<Path>,
        F: FileSystem,
    {
        let root = root.as_ref();
        let _ = self.add_folder_content_internal(filesystem, root, root, true);
        self
    }

    pub fn add_folder<F, P>(&mut self, filesystem: &F, folder: P) -> &mut Self
    where
        P: AsRef<Path>,
        F: FileSystem,
    {
        let folder = folder.as_ref();
        let _ = self.add_folder_content_internal(filesystem, folder, folder, false);
        self
    }

    pub fn add_file<F, P>(&mut self, filesystem: &F, file: P) -> &mut Self
    where
        P: AsRef<Path>,
        F: FileSystem,
    {
        let file = file.as_ref();
        let _ = self.add_file_internal(filesystem, file);
        self
    }

    fn add_file_internal<F>(&mut self, filesystem: &F, file: &Path) -> Option<()>
    where
        F: FileSystem,
    {
        if !filesystem.is_file(file) {
            return None;
        }

        let full_name = file.fullname()?;
        let file_time = filesystem.get_filetime(file).unwrap_or((0, 0));

        let data = filesystem.read_file(file).ok()?;

        let entry = ZipEntry {
            path: full_name.to_string(),
            data,
            modified: filetime_to_dos_date_time(&file_time),
        };

        self.entries.push(entry);

        Some(())
    }

    fn add_folder_content_internal<F>(
        &mut self,
        filesystem: &F,
        root: &Path,
        file: &Path,
        use_parent: bool,
    ) -> Option<()>
    where
        F: FileSystem,
    {
        if !filesystem.is_exists(file) || !filesystem.is_exists(root) {
            return None;
        }

        for file in &filesystem.list_files(file)? {
            if filesystem.is_dir(file) {
                self.add_folder_content_internal(filesystem, root, file, use_parent)?
            } else if filesystem.is_file(file) {
                let data = filesystem.read_file(file).ok()?;
                let file_time = filesystem.get_filetime(file).unwrap_or((0, 0));

                let rel_path = if use_parent {
                    file.strip_prefix(root.deref())?.strip_prefix("\\")?
                } else {
                    file.deref()
                };

                let entry = ZipEntry {
                    path: rel_path.to_string(),
                    data,
                    modified: filetime_to_dos_date_time(&file_time),
                };

                self.entries.push(entry);
            }
        }

        Some(())
    }

    pub fn get_password(&self) -> Option<Arc<str>> {
        self.password.clone()
    }

    pub fn get_comment(&self) -> Option<Arc<str>> {
        self.comment.clone()
    }

    pub fn create(&self) -> Vec<u8> {
        create_zip(self)
    }
}

fn filetime_to_dos_date_time(file_time: &(u32, u32)) -> (u16, u16) {
    let mut sys_time: SYSTEMTIME = unsafe { zeroed() };
    let file_time = FILETIME {
        dwLowDateTime: file_time.0,
        dwHighDateTime: file_time.1,
    };

    unsafe {
        if FileTimeToSystemTime(&file_time, &mut sys_time) == 0 {
            return (0, 0);
        }
    }

    let dos_time: u16 = (sys_time.wHour << 11) | (sys_time.wMinute << 5) | (sys_time.wSecond / 2);

    let year = sys_time.wYear as i32;
    let dos_date: u16 = (((year - 1980) as u16) << 9) | sys_time.wMonth << 5 | sys_time.wDay;

    (dos_time, dos_date)
}
