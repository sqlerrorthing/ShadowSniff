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
