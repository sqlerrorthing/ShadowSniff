use alloc::borrow::ToOwned;
use crate::WideString;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::mem::zeroed;
use core::ops::{Deref, Div};
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use windows_sys::core::PWSTR;
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, S_OK};
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS};
use windows_sys::Win32::Storage::FileSystem::{CopyFileW, CreateDirectoryW, CreateFileW, DeleteFileW, FindClose, FindFirstFileW, FindNextFileW, GetFileAttributesW, GetFileSizeEx, ReadFile, RemoveDirectoryW, WriteFile, CREATE_ALWAYS, CREATE_NEW, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, INVALID_FILE_ATTRIBUTES, OPEN_EXISTING, WIN32_FIND_DATAW};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Environment::GetCurrentDirectoryW;
use windows_sys::Win32::UI::Shell::{FOLDERID_RoamingAppData, FOLDERID_System, SHGetKnownFolderPath};
use obfstr::obfstr as s;

#[derive(Clone)]
pub struct Path {
    inner: String
}

impl Path {
    pub fn new<S>(path: S) -> Self
    where S: AsRef<str>
    {
        let path = path.as_ref().to_string().replace('/', "\\");
        let mut normalized = String::with_capacity(path.len());

        let mut chars = path.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                normalized.push('\\');
                while let Some(&'\\') = chars.peek() {
                    chars.next();
                }
            } else {
                normalized.push(c)
            }
        }

        Self {
            inner: normalized
        }
    }

    pub fn get_attributes(&self) -> Option<u32> {
        unsafe {
            let attr = GetFileAttributesW(self.to_wide().as_ptr());
            if attr == INVALID_FILE_ATTRIBUTES {
                None
            } else {
                Some(attr)
            }
        }
    }

    pub fn as_absolute(&self) -> Path {
        let current_dir = get_current_directory();

        let trimmed = self.inner.trim_start_matches(['\\', '/'].as_ref());
        let full = format!("{}\\{}", current_dir, trimmed);

        Path::new(full)
    }

    pub fn is_exists(&self) -> bool {
        self.get_attributes().is_some()
    }

    pub fn is_dir(&self) -> bool {
        match self.get_attributes() {
            Some(attr) => (attr & FILE_ATTRIBUTE_DIRECTORY) != 0,
            None => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self.get_attributes() {
            Some(attr) => (attr & FILE_ATTRIBUTE_DIRECTORY) == 0,
            None => false,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.inner
            .rsplit('\\')
            .next()
            .map(|s| s.rsplit_once('.').map(|(name, _)| name).unwrap_or(s))
    }
    
    pub fn fullname(&self) -> Option<&str> {
        self.inner
            .rsplit('\\')
            .next()
    }

    pub fn extension(&self) -> Option<&str> {
        self.inner
            .rsplit('\\')
            .next()?
            .rsplit_once('.')?
            .1
            .into()
    }
    
    pub fn name_and_extension(&self) -> Option<(&str, Option<&str>)> {
        let last_component = self.inner.rsplit('\\').next()?;
        
        match last_component.rsplit_once('.') {
            Some((name, ext)) if !name.is_empty() => Some((name, Some(ext))),
            _ => Some((last_component, None))
        }
    }
    
    pub fn parent(&self) -> Option<Path> {
        if let Some(pos) = self.inner.rfind('\\') {
            if pos == 0 {
                Some(Path { inner: self.inner[..=pos].to_string() })
            } else {
                Some(Path { inner: self.inner[..pos].to_string() })
            }
        } else {
            None
        }
    }
    
    #[inline]
    pub fn mkdirs(&self) -> Result<(), String> {
        mkdirs(self)
    }

    #[inline]
    pub fn mkdir(&self) -> Result<(), String> {
        mkdir(self)
    }

    #[inline]
    pub fn remove_dir_contents(&self) -> Result<(), String> {
        remove_dir_contents(self)
    }

    #[inline]
    pub fn read_file(&self) -> Result<Vec<u8>, String> {
        read_file(self)
    }

    #[inline]
    pub fn remove_dir(&self) -> Result<(), String> {
        remove_dir(self)
    }

    #[inline]
    pub fn remove_dir_all(&self) -> Result<(), String> {
        remove_dir_all(self)
    }

    #[inline]
    pub fn remove_file(&self) -> Result<(), String> {
        remove_file(self)
    }

    #[inline]
    pub fn create_file(&self) -> Result<(), String> {
        create_file(self)
    }

    #[inline]
    pub fn write_file(&self, data: &[u8]) -> Result<(), String> {
        write_file(self, data)
    }
    
    #[inline]
    pub fn list_files(&self) -> Option<Vec<Path>> {
        list_files(self)
    }

    #[inline]
    pub fn copy_content<F>(&self, dst: &Path, filter: &F) -> Result<(), String>
    where
        F: Fn(&Path) -> bool
    {
        copy_content(self, dst, filter)
    }

    #[inline]
    pub fn copy_all_content(&self, dst: &Path) -> Result<(), String> {
        copy_all_content(self, dst)
    }
    
    #[inline]
    pub fn copy_folder_with_filter<F>(&self, dst: &Path, filter: &F) -> Result<(), String>
    where
        F: Fn(&Path) -> bool
    {
        copy_folder_with_filter(self, dst, filter)
    }
    
    #[inline]
    pub fn copy_folder(&self, dst: &Path) -> Result<(), String> {
        copy_folder(self, dst)
    }
    
    #[inline]
    pub fn copy_file(&self, dst: &Path, with_name: bool) -> Result<(), String> {
        copy_file(self, dst, with_name)
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &str {
        &self.inner
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<S> Div<S> for &Path
where
    S: AsRef<str>
{
    type Output = Path;

    fn div(self, rhs: S) -> Self::Output {
        let rhs_str = rhs.as_ref().replace('/', "\\");
        let mut new_path = self.inner.clone();

        if !new_path.ends_with('\\') {
            new_path.push('\\');
        }

        new_path.push_str(&rhs_str);

        Path::new(new_path)
    }
}

impl<S> Div<S> for Path
where
    S: AsRef<str>
{
    type Output = Path;
    
    fn div(self, rhs: S) -> Self::Output {
        &self / rhs
    }
}

pub trait WriteToFile {
    fn write_to(self, path: &Path) -> Result<(), String>;
}

impl<T> WriteToFile for T
where T: AsRef<[u8]>
{
    fn write_to(self, path: &Path) -> Result<(), String> {
        path.write_file(self.as_ref())
    }
}

pub fn write_file(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.is_exists() {
            if let Err(e) = parent.mkdirs() {
                return Err(e);
            }
        }
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
            null_mut()
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err(format!("Failed to get file handle to write file '{}', error code: {}", path, GetLastError()))
        }

        let mut bytes_written: u32 = 0;

        let result = WriteFile(
            handle,
            data.as_ptr() as *const _,
            data.len() as u32,
            &mut bytes_written,
            null_mut()
        );

        CloseHandle(handle);

        if result == FALSE {
            return Err(format!("Failed to write file file '{}', error code: {}", path, GetLastError()))
        }

        if bytes_written as usize != data.len() {
            return Err(format!("Failed to write all bytes to file '{}'", path))
        }
    }

    Ok(())
}

pub fn list_files(path: &Path) -> Option<Vec<Path>> {
    let search_path = if path.ends_with('\\') {
        format!("{}*", path)
    } else {
        format!("{}\\*", path)
    };

    let wide_search = search_path.to_wide();
    
    unsafe {
        let mut data: WIN32_FIND_DATAW = zeroed();

        let handle = FindFirstFileW(wide_search.as_ptr(), &mut data);
        if handle == INVALID_HANDLE_VALUE {
            return None
        }

        let mut results = Vec::new();

        loop {
            let name = String::from_utf16_lossy(
                &data.cFileName[.. {
                    let mut len = 0;
                    while len < data.cFileName.len() && data.cFileName[len] != 0 {
                        len += 1;
                    }
                    len
                }]
            );

            if name != "." && name != ".." {
                let full_path = path / name;
                results.push(full_path);
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

pub fn copy_all_content(src: &Path, dst: &Path) -> Result<(), String> {
    copy_content(src, dst, &|_| true)
}

pub fn copy_content<F>(src: &Path, dst: &Path, filter: &F) -> Result<(), String>
where
    F: Fn(&Path) -> bool
{
    if !src.is_dir() {
        return Err(s!("Source must be a directory").to_owned())
    }
    
    if let Some(files) = list_files(src) {
        for entry in files {
            if !filter(&entry) {
                continue
            }
            
            let relative = entry.inner.strip_prefix(&src.inner)
                .ok_or(s!("Failed to compute relative path").to_owned())?;
            
            let new_dst = dst / relative;
            
            if entry.is_dir() {
                copy_content(&entry, &new_dst, filter)?;
            } else {
                copy_file(&entry, &new_dst, false)?;
            }
        }
    }
    
    Ok(())
}

pub fn copy_folder(src: &Path, dst: &Path) -> Result<(), String> {
    copy_folder_with_filter(src, dst, &|_| true)
}

pub fn copy_folder_with_filter<F>(src: &Path, dst: &Path, filter: &F) -> Result<(), String>
where
    F: Fn(&Path) -> bool
{
    if !src.is_dir() {
        return Err(s!("Source must be a directory").to_owned())
    }

    let dst = dst / src.name().ok_or(s!("Failed to get folder name"))?;
    copy_content(src, &dst, filter)?;

    Ok(())
}

pub fn copy_file(src: &Path, dst: &Path, with_filename: bool) -> Result<(), String> {
    if !src.is_file() {
        return Err(s!("Source must be a file").to_owned())
    }
    
    let dst = if with_filename {
        &(dst / src.fullname().ok_or(s!("Failed to get file name"))?)
    } else {
        dst
    };
    
    let src_wide = src.to_wide();
    let dst_wide = dst.to_wide();

    if let Some(parent) = dst.parent() {
        if !parent.is_exists() {
            parent.mkdirs()?;
        }
    }

    unsafe {
        if CopyFileW(
            src_wide.as_ptr(),
            dst_wide.as_ptr(),
            FALSE
        ) == 0 {
            return Err(format!("Failed to copy file {} (to {}), error code: {}", src, dst, GetLastError()))
        }
    }
    
    Ok(())
}

pub fn remove_dir_contents(path: &Path) -> Result<(), String> {
    if let Some(entries) = list_files(path) {
        for entry in entries {
            let is_dir = entry.is_dir();

            if is_dir {
                remove_dir_all(&entry)?;
            } else {
                remove_file(&entry)?;
            }
        }
    }
    
    Ok(())
}

pub fn remove_dir_all(path: &Path) -> Result<(), String> {
    remove_dir_contents(path)?;
    remove_dir(path)?;
    Ok(())
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    let wide = path.to_wide();

    unsafe {
        let handle = CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            0,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut()
        );

        if handle == INVALID_HANDLE_VALUE {
            let err = GetLastError();
            return Err(format!("Failed to open file {}, error code: {}", path, err));
        }

        let mut size: i64 = zeroed();
        if GetFileSizeEx(handle, &mut size) == 0 {
            CloseHandle(handle);
            return Err(s!("Failed to get file size").to_owned())
        }

        let file_size = size as usize;
        let mut buffer: Vec<u8> = Vec::with_capacity(file_size);
        buffer.set_len(file_size);

        let mut bytes_read = 0;
        let read_ok = ReadFile(
            handle,
            buffer.as_mut_ptr() as *mut _,
            file_size as _,
            &mut bytes_read,
            null_mut()
        );

        CloseHandle(handle);

        if read_ok == 0 {
            let err = GetLastError();
            return Err(format!("Failed to read from file {}, error code: {}", path, err));
        }

        buffer.truncate(bytes_read as usize);
        Ok(buffer)
    }
}

pub fn remove_dir(path: &Path) -> Result<(), String> {
    unsafe {
        if RemoveDirectoryW(path.to_wide().as_ptr()) == 0 {
            let err = GetLastError();
            Err(format!("Failed to remove directory '{}', error code: {}", path, err))
        } else {
            Ok(())
        }
    }
}

pub fn remove_file(path: &Path) -> Result<(), String> {
    unsafe {
        if DeleteFileW(path.to_wide().as_ptr()) == 0 {
            let err = GetLastError();
            Err(format!("Failed to delete file '{}', error code: {}", path, err))
        } else {
            Ok(())
        }
    }
}

pub fn create_file(path: &Path) -> Result<(), String> {
    let wide = path.to_wide();
    unsafe {
        let handle = CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE | GENERIC_READ,
            0,
            null_mut(),
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            null_mut()
        );

        if handle == INVALID_HANDLE_VALUE {
            let err = GetLastError();

            return if err == ERROR_FILE_EXISTS {
                Ok(())
            } else {
                Err(format!("Failed to create file {}, error code: {}", path, err))
            }
        }

        CloseHandle(handle);
    }

    Ok(())
}

pub fn mkdir(path: &Path) -> Result<(), String> {
    let wide = path.to_wide();

    unsafe {
        let success = CreateDirectoryW(wide.as_ptr(), null_mut());
        if success == 0 {
            let err = GetLastError();
            if err != ERROR_ALREADY_EXISTS {
                return Err(format!("Failed to create directory: {}, err: {}", path, err));
            }
        }
    }

    Ok(())
}

pub fn mkdirs(path: &Path) -> Result<(), String> {
    let parts: Vec<&str> = path
        .split('\\')
        .filter(|part| !part.is_empty())
        .collect();

    let mut current = String::new();

    for part in parts {
        if !current.is_empty() {
            current.push('\\');
        }

        current.push_str(part);

        let subpath = Path::new(&current);

        mkdir(&subpath)?;
    }

    Ok(())
}

pub fn get_current_directory() -> Path {
    let required_size = unsafe { GetCurrentDirectoryW(0, null_mut()) };
    if required_size == 0 {
        panic!("{}", s!("Couldn't get current directory, required size is 0"));
    }

    let mut buffer: Vec<u16> = Vec::with_capacity(required_size as usize);
    unsafe { buffer.set_len(required_size as usize); }

    let len = unsafe { GetCurrentDirectoryW(required_size, buffer.as_mut_ptr()) };
    if len == 0 || len > required_size {
        panic!("{}", s!("Couldn't get current directory, len is 0 or len > required_size"));
    }

    unsafe { buffer.set_len(len as usize) };

    Path::new(String::from_utf16(&buffer).expect(s!("Couldn't get current directory")))
}

pub fn get_known_folder_path(folder_id: &windows_sys::core::GUID) -> Option<Path> {
    unsafe {
        let mut path_raw_ptr: PWSTR = null_mut();
        let hr = SHGetKnownFolderPath(folder_id, 0, null_mut(), &mut path_raw_ptr);
        if hr == S_OK {
            let mut len = 0;
            while *path_raw_ptr.add(len) != 0 {
                len += 1;
            }

            let path = String::from_utf16_lossy(from_raw_parts(path_raw_ptr, len));

            CoTaskMemFree(path_raw_ptr as _);
            Some(Path::new(path))
        } else {
            None
        }
    }
}

impl Path {
    pub fn system() -> Self {
        get_known_folder_path(&FOLDERID_System).unwrap()
    }
    
    pub fn appdata() -> Self {
        get_known_folder_path(&FOLDERID_RoamingAppData).unwrap()
    }
}