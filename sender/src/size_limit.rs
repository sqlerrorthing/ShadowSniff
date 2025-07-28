use crate::{ExternalLink, LogContent, LogFile, LogSender, SendError};
use collector::Collector;
use derive_new::new;

/// A wrapper around a [`LogSender`] that enforces a maximum log file size.
///
/// This sender checks the size of the provided log before delegating to the inner sender.
/// If the log exceeds the configured `max_size_bytes`, it returns a [`SendError::LogFileTooBig`] instead of sending.
///
/// # Fields
///
/// - `inner`: The inner sender used when size checks pass.
/// - `max_size_bytes`: Maximum allowed size in bytes.
/// - `check_external_link_size`: If `true`, also checks size for [`LogFile::ExternalLink`] variants.
#[derive(new, Clone)]
pub struct SizeLimitWrapper<T: LogSender> {
    inner: T,
    max_size_bytes: usize,
    check_external_link_size: bool,
}

impl<T: LogSender> LogSender for SizeLimitWrapper<T> {
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
            LogContent::ZipArchive(data) if data.len() > self.max_size_bytes => {
                Err(SendError::LogFileTooBig)
            }
            LogContent::ExternalLink(ExternalLink { size, .. })
                if self.check_external_link_size && *size > self.max_size_bytes =>
            {
                Err(SendError::LogFileTooBig)
            }
            _ => self.inner.send(log_file, password, collector),
        }
    }
}
