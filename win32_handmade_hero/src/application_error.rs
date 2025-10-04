use std::error::Error;

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ApplicationError {
    message: String,
    #[source]
    source: Option<Box<dyn Error>>,
}

impl ApplicationError {
    #[inline]
    #[must_use]
    pub fn wrap(message: impl Into<String>, source: impl Error + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

pub type Result<T> = std::result::Result<T, ApplicationError>;
