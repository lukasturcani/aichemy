//! Make AI in chemistry simple!
//!
//! The goal of this crate is to make it easy for you to manage your chemical
//! data and use it for AI workflows. This includes both experimental and
//! computational data.
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]

use thiserror::Error;

pub mod nmr;

/// A library error.
#[derive(Error, Debug)]
pub enum Error {
    /// Error reading or writing a file.
    #[error("{message}")]
    Io {
        /// Message describing the error.
        message: String,
        /// The underlying error.
        source: Option<anyhow::Error>,
    },
    /// A NOMAD NMR error.
    #[error("NOMAD NMR error")]
    NomadNmrError(#[from] nmr::nomad_nmr::Error),
    /// An error with an NMR spectrum.
    #[error("{message}")]
    NmrError {
        /// Message describing the error.
        message: String,
    },
}
