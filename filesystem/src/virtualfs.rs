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

use crate::FileSystem;
use crate::path::Path;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::RwLock;

#[derive(Clone)]
enum Entry {
    File { data: Vec<u8> },
    Directory,
}

pub struct VirtualFileSystem {
    entries: RwLock<BTreeMap<String, Entry>>,
}

impl AsRef<Self> for VirtualFileSystem {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        let mut map = BTreeMap::new();

        map.insert(String::from("\\"), Entry::Directory);

        Self {
            entries: RwLock::new(map),
        }
    }
}

fn parent_path(s: &str) -> Option<String> {
    s.rfind('\\').map(|pos| {
        if pos == 0 {
            String::from("\\")
        } else {
            s[..pos].to_string()
        }
    })
}

impl FileSystem for VirtualFileSystem {
    fn read_file<P>(&self, path: P) -> Result<Vec<u8>, u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let map = self.entries.read();

        match map.get(&path.to_string()) {
            Some(Entry::File { data, .. }) => Ok(data.clone()),
            Some(Entry::Directory) => Err(1), // error: is a directory
            None => Err(2),                   // error: not found
        }
    }

    fn mkdir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        if map.contains_key(&path_str) {
            return Err(3); // already exists
        }

        // Parent must exist and be a directory
        let parent = parent_path(&path_str).ok_or(4u32)?; // error no parent

        match map.get(&parent) {
            Some(Entry::Directory) => (),
            _ => return Err(5), // parent not dir or not found
        }

        map.insert(path_str, Entry::Directory);
        Ok(())
    }

    fn mkdirs<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        // Iterate through each directory component and create if missing
        let mut components = path_str.split('\\').filter(|s| !s.is_empty());

        let mut current_path = String::from("\\");
        while let Some(comp) = components.next() {
            if current_path != "\\" {
                current_path.push('\\');
            }
            current_path.push_str(comp);

            if !map.contains_key(&current_path) {
                map.insert(current_path.clone(), Entry::Directory);
            } else if let Some(Entry::File { .. }) = map.get(&current_path) {
                return Err(6); // a file exists where a directory should be
            }
        }
        Ok(())
    }

    fn remove_dir_contents<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        let prefix = if path_str.ends_with('\\') {
            path_str.clone()
        } else {
            format!("{path_str}\\")
        };

        let to_remove: Vec<_> = map
            .keys()
            .filter(|k| k != &&path_str && k.starts_with(&prefix))
            .cloned()
            .collect();

        for k in to_remove {
            map.remove(&k);
        }

        Ok(())
    }

    fn remove_dir<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        // Only remove directory if empty (no entries with prefix)
        let prefix = if path_str.ends_with('\\') {
            path_str.clone()
        } else {
            format!("{path_str}\\")
        };

        if map.keys().any(|k| k != &path_str && k.starts_with(&prefix)) {
            return Err(7); // directory not empty
        }

        match map.get(&path_str) {
            Some(Entry::Directory) => {
                map.remove(&path_str);
                Ok(())
            }
            _ => Err(8), // not a directory or does not exist
        }
    }

    fn remove_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        match map.get(&path_str) {
            Some(Entry::File { .. }) => {
                map.remove(&path_str);
                Ok(())
            }
            Some(Entry::Directory) => Err(9), // is a directory
            None => Err(2),                   // not found
        }
    }

    fn create_file<P>(&self, path: P) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let mut map = self.entries.write();

        if map.contains_key(&path_str) {
            return Err(3); // already exists
        }

        // Parent directory must exist
        let parent = parent_path(&path_str).ok_or(4u32)?;

        match map.get(&parent) {
            Some(Entry::Directory) => (),
            _ => return Err(5), // parent not dir or missing
        }

        map.insert(path_str, Entry::File { data: Vec::new() });
        Ok(())
    }

    fn write_file<P>(&self, path: P, data: &[u8]) -> Result<(), u32>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !self.is_exists(&parent)
        {
            self.mkdirs(parent)?
        }

        let path_str = path.to_string();
        let mut map = self.entries.write();

        match map.get_mut(&path_str) {
            Some(Entry::File {
                data: existing_data,
                ..
            }) => {
                existing_data.clear();
                existing_data.extend_from_slice(data);
                Ok(())
            }
            Some(Entry::Directory) => Err(9), // is directory
            None => {
                let parent = parent_path(&path_str).ok_or(4u32)?;

                match map.get(&parent) {
                    Some(Entry::Directory { .. }) => {
                        // Insert new file
                        map.insert(
                            path_str,
                            Entry::File {
                                data: data.to_vec(),
                            },
                        );
                        Ok(())
                    }
                    _ => Err(5), // parent not directory or missing
                }
            }
        }
    }

    fn list_files_filtered<F, P>(&self, path: P, filter: &F) -> Option<Vec<Path>>
    where
        F: Fn(&Path) -> bool,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let dir_str = path.to_string();
        let map = self.entries.read();

        let prefix = if dir_str.ends_with('\\') {
            dir_str.clone()
        } else {
            format!("{dir_str}\\")
        };

        if !map.contains_key(&dir_str) {
            return None;
        }
        match map.get(&dir_str)? {
            Entry::Directory { .. } => {}
            _ => return None,
        }

        let mut seen = BTreeMap::new(); // to collect unique immediate children
        for key in map.keys() {
            if key.starts_with(&prefix) {
                // strip prefix:
                let remainder = &key[prefix.len()..];

                // immediate child = next component before next '\'
                if let Some(pos) = remainder.find('\\') {
                    let child = &remainder[..pos];
                    seen.entry(child.to_string()).or_insert(true);
                } else {
                    // remainder itself is an immediate child (file or dir)
                    seen.entry(remainder.to_string()).or_insert(true);
                }
            }
        }

        let results = seen
            .keys()
            .map(|child_name| {
                // build full child path
                let full_path = if dir_str == "\\" {
                    format!("\\{child_name}")
                } else {
                    format!("{dir_str}\\{child_name}")
                };
                Path::new(full_path)
            })
            .filter(|p| filter(p))
            .collect();

        Some(results)
    }

    fn get_filetime<P>(&self, _: P) -> Option<(u32, u32)>
    where
        P: AsRef<Path>,
    {
        None
    }

    fn is_exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let map = self.entries.read();
        map.contains_key(&path_str)
    }

    fn is_dir<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let map = self.entries.read();
        matches!(map.get(&path_str), Some(Entry::Directory))
    }

    fn is_file<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_str = path.to_string();
        let map = self.entries.read();
        matches!(map.get(&path_str), Some(Entry::File { .. }))
    }
}
