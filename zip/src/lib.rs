#![no_std]

extern crate alloc;
mod create;

use crate::create::create_zip;
use alloc::string::{String, ToString};
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

pub enum IntoPath<'a, 'b> {
    Reference(&'a Path),
    Borrowed(Path),
    StringReference(&'b str),
}

impl From<IntoPath<'_, '_>> for Path {
    fn from(value: IntoPath) -> Self {
        match value {
            IntoPath::Reference(val) => val.clone(),
            IntoPath::Borrowed(val) => val,
            IntoPath::StringReference(val) => Path::new(val),
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
    pub fn comment<S>(&mut self, comment: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.comment = Some(comment.as_ref().to_string());
        self
    }

    pub fn password<S>(&mut self, password: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        assert!(password.as_ref().is_ascii(), "Password must be ASCII only");
        self.password = Some(password.as_ref().to_string());
        self
    }

    pub fn compression(&mut self, compression: ZipCompression) -> &mut Self {
        self.compression = compression;
        self
    }

    pub fn add_folder_content<'a, 'b, F, P>(&mut self, filesystem: &F, root: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
        F: FileSystem,
    {
        let root = &Path::from(root.into());
        let _ = self.add_folder_content_internal(filesystem, root, root, true);
        self
    }

    pub fn add_folder<'a, 'b, F, P>(&mut self, filesystem: &F, folder: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
        F: FileSystem,
    {
        let folder = &Path::from(folder.into());
        let _ = self.add_folder_content_internal(filesystem, folder, folder, false);
        self
    }

    pub fn add_file<'a, 'b, F, P>(&mut self, filesystem: &F, file: P) -> &mut Self
    where
        P: Into<IntoPath<'a, 'b>>,
        F: FileSystem,
    {
        let file = &Path::from(file.into());
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
