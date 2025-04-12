use std::sync::atomic::{AtomicBool, Ordering};

use derive_more::{Display, From};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    #[from]
    Custom(String),

    #[display("Error: {}\n{} | {}\n       ^-- Here.", message, line, location)]
    ParsingError{
        line: usize, 
        location: String,
        message: String,
    },

    #[from]
    StdIoError(std::io::Error)
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}

impl std::error::Error for Error {}

// TODO: non mi piace la flag globale
static HAD_ERROR: AtomicBool = AtomicBool::new(false);

/// Reports an error by printing it (using our Display format) and sets the error flag.
pub fn report(error: Error) {
    eprintln!("{}", error);
    HAD_ERROR.store(true, Ordering::SeqCst);
}

/// Returns whether an error has been reported.
pub fn had_error() -> bool {
    HAD_ERROR.load(Ordering::SeqCst)
}

/// Resets the error flag.
pub fn reset() {
    HAD_ERROR.store(false, Ordering::SeqCst);
}