use lox_syntax::Token;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InterpretError { message: String },

    RuntimeError { token: Token, message: String },
}

impl Error {
    pub fn interpret_error(message: impl std::fmt::Display) -> Self {
        Self::InterpretError {
            message: message.to_string(),
        }
    }

    pub fn runtime_error(token: Token, message: impl std::fmt::Display) -> Self {
        Self::RuntimeError {
            token,
            message: message.to_string(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
