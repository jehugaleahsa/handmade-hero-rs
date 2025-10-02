use std::error::Error;

/// Represents an error that occurred while reading a SAS® transport file.
#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ApplicationError {
    message: String,
    #[source]
    source: Option<Box<dyn Error>>,
}

impl ApplicationError {
    /// Creates a new error with the given message, wrapping the given source.
    /// Use this method when wrapping an internal error.
    #[inline]
    #[must_use]
    pub fn wrap(message: impl Into<String>, source: impl Error + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// A result that returns an `XportError` when an error occurs.
pub type Result<T> = std::result::Result<T, ApplicationError>;
