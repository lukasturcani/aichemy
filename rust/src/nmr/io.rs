//! Tools for NMR I/O, such a reading and writing files.

use std::io;

use thiserror::Error;

pub mod jcamp_dx;

/// Error which may occur when doing NMR I/O.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to parse a file.
    #[error("Failed to parse file: {0}")]
    Parse(String),

    /// Failed to read a file.
    #[error("Failed to read file: {source}")]
    Read { source: io::Error },

    /// Failed to write a file.
    #[error("Failed to write file: {0}")]
    Write(String),
}
