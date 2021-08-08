use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct CustomError {
    msg: String,
}

impl CustomError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for CustomError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl StdError for CustomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

