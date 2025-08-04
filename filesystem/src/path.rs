/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::fmt::{Display, Formatter};
use core::iter::once;
use core::ops::{Deref, Div};
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use windows_sys::Win32::Foundation::S_OK;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Environment::GetCurrentDirectoryW;
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::UI::Shell::{
    FOLDERID_LocalAppData, FOLDERID_RoamingAppData, FOLDERID_System, SHGetKnownFolderPath,
};
use windows_sys::core::PWSTR;

#[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct Path {
    inner: Arc<str>,
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        let path = value.replace('/', "\\");
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
            inner: normalized.into(),
        }
    }
}

impl Path {
    pub fn new<S>(path: S) -> Self
    where
        S: AsRef<str>,
    {
        path.as_ref().to_string().into()
    }

    pub fn as_absolute(&self) -> Path {
        let current_dir = get_current_directory().unwrap();

        let trimmed = self.inner.trim_start_matches(['\\', '/'].as_ref());
        let full = format!("{current_dir}\\{trimmed}");

        Path::new(full)
    }

    pub fn name(&self) -> Option<&str> {
        self.inner
            .rsplit('\\')
            .next()
            .map(|s| s.rsplit_once('.').map(|(name, _)| name).unwrap_or(s))
    }

    pub fn fullname(&self) -> Option<&str> {
        self.inner.rsplit('\\').next()
    }

    pub fn extension(&self) -> Option<&str> {
        self.inner.rsplit('\\').next()?.rsplit_once('.')?.1.into()
    }

    pub fn name_and_extension(&self) -> Option<(&str, Option<&str>)> {
        let last_component = self.inner.rsplit('\\').next()?;

        match last_component.rsplit_once('.') {
            Some((name, ext)) if !name.is_empty() => Some((name, Some(ext))),
            _ => Some((last_component, None)),
        }
    }

    pub fn parent(&self) -> Option<Path> {
        if let Some(pos) = self.inner.rfind('\\') {
            if pos == 0 {
                Some(Path {
                    inner: self.inner[..=pos].into(),
                })
            } else {
                Some(Path {
                    inner: self.inner[..pos].into(),
                })
            }
        } else {
            None
        }
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
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
    S: AsRef<str>,
{
    type Output = Path;

    fn div(self, rhs: S) -> Self::Output {
        let rhs_str = rhs.as_ref().replace('/', "\\");
        let mut new_path = self.inner.to_string();

        if !new_path.ends_with('\\') {
            new_path.push('\\');
        }

        new_path.push_str(&rhs_str);

        Path::new(Arc::from(new_path))
    }
}

impl<S> Div<S> for Path
where
    S: AsRef<str>,
{
    type Output = Path;

    fn div(self, rhs: S) -> Self::Output {
        &self / rhs
    }
}

pub fn get_current_directory() -> Option<Path> {
    let required_size = unsafe { GetCurrentDirectoryW(0, null_mut()) };
    if required_size == 0 {
        return None;
    }

    let mut buffer = vec![0u16; required_size as usize];
    unsafe {
        buffer.set_len(required_size as usize);
    }

    let len = unsafe { GetCurrentDirectoryW(required_size, buffer.as_mut_ptr()) };
    if len == 0 || len > required_size {
        return None;
    }

    unsafe { buffer.set_len(len as usize) };

    Some(Path::new(String::from_utf16(&buffer).ok()?))
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

    pub fn localappdata() -> Self {
        get_known_folder_path(&FOLDERID_LocalAppData).unwrap()
    }

    pub fn temp() -> Self {
        Self::localappdata() / "Temp"
    }

    pub fn temp_file<S>(prefix: S) -> Self
    where
        S: AsRef<str>,
    {
        let ms = unsafe { GetTickCount64() };
        let name = format!("{ms:x}");
        Self::temp() / format!("{}{name}", prefix.as_ref())
    }
}

pub trait WideString {
    fn to_wide(&self) -> Vec<u16>;
}

impl WideString for Path {
    fn to_wide(&self) -> Vec<u16> {
        self.encode_utf16().chain(once(0)).collect()
    }
}
