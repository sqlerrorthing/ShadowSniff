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

#![no_std]

mod kvm_check;

extern crate alloc;

use crate::kvm_check::KVMCheck;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// A trait for detecting whether the current process is running inside a virtual machine (VM).
///
/// Implement this trait for different platforms or detection mechanisms to identify
/// if the host system is a virtualized environment.
pub trait VmDetector {
    /// Determines whether the current environment is a virtual machine.
    ///
    /// # Returns
    ///
    /// * `true` - if the process is likely running in a virtual machine.
    /// * `false` - if the process is likely running a physical machine.
    fn is_running_in_vm(&self) -> bool;
}

pub enum Check {
    KVM,
    Custom(Box<dyn VmDetector>),
}

impl VmDetector for Check {
    fn is_running_in_vm(&self) -> bool {
        match self {
            Check::KVM => KVMCheck.is_running_in_vm(),
            Check::Custom(check) => check.is_running_in_vm(),
        }
    }
}

#[inline(always)]
pub fn run_checks(checks: Vec<Check>) -> bool {
    checks.iter().any(|v| v.is_running_in_vm())
}
