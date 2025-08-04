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

use crate::{LogFile, LogSender, SendError};
use collector::Collector;
use derive_new::new;

/// A log sender that attempts to send using a primary sender and falls back to another
/// if the log file is too large.
///
/// [`SizeFallbackSender`] wraps two log senders: a `primary` and a `fallback`. When
/// sending a log file, it first tries the `primary` sender. If the primary fails
/// with [`SendError::LogFileTooBig`], it retries the operation using the `fallback` sender.
///
/// # Type Parameters
/// - `Primary`: A type that implements the [`LogSender`] trait and serves as the first attempt.
/// - `Fallback`: A type that also implements [`LogSender`] and is used when the primary fails
///   due to size constraints.
#[derive(Clone, new)]
pub struct SizeFallbackSender<Primary, Fallback>
where
    Primary: LogSender,
    Fallback: LogSender,
{
    primary: Primary,
    fallback: Fallback,
}

impl<Primary, Fallback> LogSender for SizeFallbackSender<Primary, Fallback>
where
    Primary: LogSender,
    Fallback: LogSender,
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
        match self
            .primary
            .send(log_file.clone(), password.clone(), collector)
        {
            Ok(_) => Ok(()),
            Err(SendError::LogFileTooBig) => self.fallback.send(log_file, password, collector),
            Err(e) => Err(e),
        }
    }
}
