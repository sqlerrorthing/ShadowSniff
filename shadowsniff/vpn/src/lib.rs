#![no_std]

mod openvpn;

use crate::alloc::borrow::ToOwned;
extern crate alloc;

use tasks::Task;
use alloc::vec;
use tasks::{composite_task, impl_composite_task_runner, CompositeTask};
use crate::openvpn::OpenVPN;

pub struct VpnTask {
    inner: CompositeTask
}

impl VpnTask {
    pub fn new() -> Self {
        Self {
            inner: composite_task!(
                OpenVPN
            )
        }
    }
}

impl_composite_task_runner!(VpnTask, "Vpn");