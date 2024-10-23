//! Tools for NMR I/O, such a reading and writing files.
use thiserror::Error;

pub mod jcamp_dx;

/// Error which may occur when doing NMR I/O.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Failed to parse a file.
    #[error("Failed to parse file: {0}")]
    Parse(String),
}
