//! Log-to-file helper. Port of `logging.go`.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// Redirect the default `log` crate output to a file.
///
/// Analogue of `tea.LogToFile`. Returns the open file handle; close it when
/// the program exits.
///
/// # Example
/// ```no_run
/// let _f = bubbletea_rs::logging::log_to_file("debug.log").unwrap();
/// ```
pub fn log_to_file(path: impl AsRef<Path>) -> io::Result<std::fs::File> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    Ok(file)
}

/// A simple file-backed writer that can be used with the `log` crate or any
/// other logger.
pub struct FileLogger(std::fs::File);

impl Write for FileLogger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl FileLogger {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        log_to_file(path).map(FileLogger)
    }
}
