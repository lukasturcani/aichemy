//! Tools to interact with Bruker data.

use std::{fs, path::Path};

use super::io::Error;

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
    let array = fs::read(path).map_err(|source| Error::Read { source })?;
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
