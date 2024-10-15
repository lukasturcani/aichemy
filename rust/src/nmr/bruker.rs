//! Tools to interact with Bruker data.

use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not find Bruker data in {0}")]
    NotFound(PathBuf),
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
        .ok_or_else(|| Error::NotFound(dir.as_ref().into()))?;

    Ok(())
}
