//! Error types. Port of Go's sentinel errors (`ErrProgramPanic`, etc.).

use thiserror::Error;

/// Errors returned by [`crate::Program::run`].
#[derive(Debug, Error)]
pub enum Error {
    /// The program recovered from a panic.
    #[error("program experienced a panic")]
    Panic,
    /// The program was killed.
    #[error("program was killed")]
    Killed,
    /// The program was interrupted (e.g. ctrl+c or SIGINT).
    #[error("program was interrupted")]
    Interrupted,
    /// An IO error from the terminal or output.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;
