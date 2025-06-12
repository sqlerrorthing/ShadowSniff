#![no_std]

extern crate alloc;
mod create;

use crate::create::create_zip;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ops::Deref;
use miniz_oxide::deflate::compress_to_vec;
use utils::path::Path;
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

pub enum IntoPath<'a, 'b>
{
    Reference(&'a Path),
    Borrowed(Path),
    StringReference(&'b str)
}

impl From<IntoPath<'_, '_>> for Path {
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

#[derive(Copy, Clone)]
pub enum ZipCompression {
    NONE,

    DEFLATE(u8)
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
        S: AsRef<str>
    {
        self.comment = Some(comment.as_ref().to_string());
        self
    }
    
    pub fn password<S>(&mut self, password: S) -> &mut Self
    where
        S: AsRef<str>
    {
        assert!(password.as_ref().is_ascii(), "Password must be ASCII only");
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
        create_zip(self)
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
