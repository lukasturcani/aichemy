use thiserror::Error;

pub mod jcamp_dx;

/// Error which may occur when doing NMR I/O.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to parse a file.
    #[error("Failed to parse file: {source}")]
    Parse {
        /// The underlying error.
        source: nom::Err<nom::error::Error<String>>,
    },
}
