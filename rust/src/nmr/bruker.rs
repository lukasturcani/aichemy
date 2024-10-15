//! Tools to interact with Bruker data.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Error which may occur when interacting with Bruker data.
#[derive(Error, Debug)]
pub enum Error {
    /// The Bruker data could not be found.
    #[error("Could not find Bruker data in {0}")]
    DataNotFound(PathBuf),
}

pub fn read_pdata(dir: impl AsRef<Path>) -> Result<(), Error> {
    let data_dirs = [
        dir.as_ref().join("1r"),
        dir.as_ref().join("2rr"),
        dir.as_ref().join("3rrr"),
    ];
    let data_dir = data_dirs
        .iter()
        .find(|d| d.exists())
        .ok_or_else(|| Error::DataNotFound(dir.as_ref().into()))?;

    Ok(())
}

fn read_procs_file(dir: impl AsRef<Path>) -> Result<(), Error> {}
