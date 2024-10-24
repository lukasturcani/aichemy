//! Tools to interact with Bruker data.

use std::{fs, path::Path};

use crate::Error;

use super::io::jcamp_dx;

enum DataType {
    Float64,
    Integer32,
}

enum Endianness {
    Little,
    Big,
}

fn read_bruker_binary_file(
    path: impl AsRef<Path>,
    dtype: DataType,
    endianness: Endianness,
) -> Result<Vec<f64>, Error> {
    let path = path.as_ref();
    let array = fs::read(path).map_err(|source| Error::Io {
        message: format!("failed to read {path:?}"),
        source: Some(source.into()),
    })?;
    match (dtype, endianness) {
        (DataType::Float64, Endianness::Little) => Ok(array
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect()),
        (DataType::Float64, Endianness::Big) => Ok(array
            .chunks_exact(8)
            .map(|chunk| f64::from_be_bytes(chunk.try_into().unwrap()))
            .collect()),
        (DataType::Integer32, Endianness::Little) => Ok(array
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()) as f64)
            .collect()),
        (DataType::Integer32, Endianness::Big) => Ok(array
            .chunks_exact(4)
            .map(|chunk| i32::from_be_bytes(chunk.try_into().unwrap()) as f64)
            .collect()),
    }
}

// fn read_1d_spectrum(
//     binary: impl AsRef<Path>,
//     procs: impl AsRef<Path>,
//     acqus: impl AsRef<Path>,
// ) -> Result<(), Error> {
//     let procs = jcamp_dx::parse(fs::read(procs).map_err(|source| Error::Read { source })?)?;
//     let acqus = jcamp_dx::parse(fs::read(acqus).map_err(|source| Error::Read { source })?)?;
//     let si = procs["$SI"];
//     let xdim = procs["$XDIM"];
//     let endianness = procs
//         .get("$BYTEORD")
//         .map_or(Endianness::Little, |byte_order| {});
//     Ok(())
// }
