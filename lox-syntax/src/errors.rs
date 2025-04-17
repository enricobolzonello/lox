use derive_more::From;

use crate::tokenizer::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Clone)]
pub enum Error {
    #[from]
    Custom(String),

    ParseError {
        token: Token,
        message: String,
    },
}

impl Error {
    pub fn parse_error(token: Token, message: impl std::fmt::Display) -> Self {
        Self::ParseError {
            token,
            message: message.to_string(),
        }
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
