use std::{error::Error, sync::atomic::{AtomicBool, Ordering}};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

// TODO: non mi piace la flag globale
static HAD_ERROR: AtomicBool = AtomicBool::new(false);

/// Reports an error by printing it (using our Display format) and sets the error flag.
pub fn report(error: Box<dyn Error>) {
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