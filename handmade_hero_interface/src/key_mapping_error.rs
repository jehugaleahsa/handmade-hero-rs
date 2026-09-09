use crate::button::Button;
use crate::key::Key;
use thiserror::Error;

#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum KeyMappingError {
    #[error("The key {key:?} is already bound to {button:?}")]
    KeyAlreadyBound { key: Key, button: Button },
}
