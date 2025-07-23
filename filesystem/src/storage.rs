use core::{mem::zeroed, ptr::null_mut};

use crate::path::{Path, WideString};
use crate::{FileSystem, FileSystemExt};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::iter::once;
use windows_sys::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, FALSE, GENERIC_WRITE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, DeleteFileW, FindClose, FindFirstFileW, FindNextFileW, GetFileAttributesExW,
    GetFileAttributesW, GetFileExInfoStandard, RemoveDirectoryW, WriteFile, CREATE_ALWAYS,
    CREATE_NEW, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES, WIN32_FILE_ATTRIBUTE_DATA,
    WIN32_FIND_DATAW,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, GENERIC_READ, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, GetFileSizeEx, ReadFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    },
};

pub struct StorageFileSystem;

impl AsRef<StorageFileSystem> for StorageFileSystem {
    fn as_ref(&self) -> &StorageFileSystem {
        self
    }
}

impl FileSystem for StorageFileSystem {
    fn read_file<P>(&self, path: P) -> Result<Vec<u8>, u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let wide = path.to_wide();

        unsafe {
            let handle = CreateFileW(
                wide.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            );

            if handle == INVALID_HANDLE_VALUE {
                return Err(GetLastError());
            }

            let mut size: i64 = zeroed();
            if GetFileSizeEx(handle, &mut size) == 0 {
                CloseHandle(handle);
                return Err(1000001);
            }

            let file_size = size as usize;
            let mut buffer: Vec<u8> = vec![0u8; file_size];
            buffer.set_len(file_size);
            let mut bytes_read = 0;

            let read_ok = ReadFile(
                handle,
                buffer.as_mut_ptr() as *mut _,
                file_size as _,
                &mut bytes_read,
                null_mut(),
            );

            CloseHandle(handle);

            if read_ok == 0 {
                return Err(GetLastError());
            }

            buffer.truncate(bytes_read as usize);
            Ok(buffer)
        }
    }

    fn mkdir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let wide = path.to_wide();

        unsafe {
            let success = CreateDirectoryW(wide.as_ptr(), null_mut());
            if success == 0 {
                let err = GetLastError();
                if err != ERROR_ALREADY_EXISTS {
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    fn mkdirs<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let parts: Vec<&str> = path.split('\\').filter(|part| !part.is_empty()).collect();

        let mut current = String::new();

        for part in parts {
            if !current.is_empty() {
                current.push('\\');
            }

            current.push_str(part);

            let subpath = Path::new(&current);

            self.mkdir(&subpath)?;
        }

        Ok(())
    }

    fn remove_dir_contents<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        if let Some(entries) = &self.list_files(path) {
            for entry in entries {
                if self.is_dir(entry) {
                    self.remove_dir_all(entry)?;
                } else {
                    self.remove_file(entry)?;
                }
            }
        }

        Ok(())
    }

    fn remove_dir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        unsafe {
            if RemoveDirectoryW(path.to_wide().as_ptr()) == 0 {
                Err(GetLastError())
            } else {
                Ok(())
            }
        }
    }

    fn remove_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        unsafe {
            if DeleteFileW(path.to_wide().as_ptr()) == 0 {
                Err(GetLastError())
            } else {
                Ok(())
            }
        }
    }

    fn create_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let wide = path.to_wide();
        unsafe {
            let handle = CreateFileW(
                wide.as_ptr(),
                GENERIC_WRITE | GENERIC_READ,
                0,
                null_mut(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            );

            if handle == INVALID_HANDLE_VALUE {
                let err = GetLastError();

                return if err == ERROR_FILE_EXISTS {
                    Ok(())
                } else {
                    Err(err)
                };
            }

            CloseHandle(handle);
        }

        Ok(())
    }

    fn write_file<P>(&self, path: P, data: &[u8]) -> Result<(), u32>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !self.is_exists(&parent)
        {
            self.mkdirs(parent)?;
        }

        let wide = path.to_wide();

        unsafe {
            let handle = CreateFileW(
                wide.as_ptr(),
                GENERIC_WRITE,
                0,
                null_mut(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            );

            if handle == INVALID_HANDLE_VALUE {
                return Err(GetLastError());
            }

            let mut bytes_written: u32 = 0;

            let result = WriteFile(
                handle,
                data.as_ptr() as *const _,
                data.len() as u32,
                &mut bytes_written,
                null_mut(),
            );

            CloseHandle(handle);

            if result == FALSE {
                return Err(GetLastError());
            }

            if bytes_written as usize != data.len() {
                return Err(GetLastError());
            }
        }

        Ok(())
    }

    fn list_files_filtered<F, P>(&self, path: P, filter: &F) -> Option<Vec<Path>>
    where
        F: Fn(&Path) -> bool,
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let search_path = if path.ends_with('\\') {
            format!("{path}*")
        } else {
            format!("{path}\\*")
        };

        let search_path: Vec<u16> = search_path.encode_utf16().chain(once(0)).collect();

        unsafe {
            let mut data: WIN32_FIND_DATAW = zeroed();

            let handle = FindFirstFileW(search_path.as_ptr(), &mut data);
            if handle == INVALID_HANDLE_VALUE {
                return None;
            }

            let mut results = Vec::new();

            loop {
                let name = String::from_utf16_lossy(
                    &data.cFileName[..{
                        let mut len = 0;
                        while len < data.cFileName.len() && data.cFileName[len] != 0 {
                            len += 1;
                        }

                        len
                    }],
                );

                if name != "." && name != ".." {
                    let full_path = path / name;

                    if filter(&full_path) {
                        results.push(full_path);
                    }
                }

                let res = FindNextFileW(handle, &mut data);
                if res == FALSE {
                    break;
                }
            }

            FindClose(handle);
            Some(results)
        }
    }

    fn get_filetime<P>(&self, path: P) -> Option<(u32, u32)>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        let mut data: WIN32_FILE_ATTRIBUTE_DATA = unsafe { zeroed() };

        if unsafe {
            GetFileAttributesExW(
                path.to_wide().as_ptr(),
                GetFileExInfoStandard,
                &mut data as *mut _ as *mut _,
            )
        } == FALSE
        {
            None
        } else {
            let write_time = data.ftLastWriteTime;
            Some((write_time.dwHighDateTime, write_time.dwLowDateTime))
        }
    }

    fn is_exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        get_attributes(path).is_some()
    }

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        if let Some(attr) = get_attributes(path)
            && (attr & FILE_ATTRIBUTE_DIRECTORY) != 0
        {
            true
        } else {
            false
        }
    }

    fn is_file<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();
        if let Some(attr) = get_attributes(path)
            && (attr & FILE_ATTRIBUTE_DIRECTORY) == 0
        {
            true
        } else {
            false
        }
    }
}

fn get_attributes(path: &Path) -> Option<u32> {
    unsafe {
        let attr = GetFileAttributesW(path.to_wide().as_ptr());
        if attr == INVALID_FILE_ATTRIBUTES {
            None
        } else {
            Some(attr)
        }
    }
}
