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

fn scale_data(data: &mut [f64]) {
    todo!()
}

fn read_1d_spectrum(
    binary: impl AsRef<Path>,
    procs: impl AsRef<Path>,
    acqus: impl AsRef<Path>,
) -> Result<(), Error> {
    let binary = binary.as_ref();
    let procs = procs.as_ref();
    let acqus = acqus.as_ref();
    let procs = jcamp_dx::parse(fs::read(procs).map_err(|source| Error::Io {
        message: format!("failed to read {procs:?}"),
        source: Some(source.into()),
    })?)?;
    let acqus = jcamp_dx::parse(fs::read(acqus).map_err(|source| Error::Io {
        source: Some(source.into()),
        message: format!("failed to read {acqus:?}"),
    })?)?;
    let si = procs
        .get("$SI")
        .ok_or(Error::NmrError {
            message: format!("$SI variable missing from {procs:?}"),
        })?
        .as_integer()
        .ok_or(Error::NmrError {
            message: format!("$SI variable is not an integer in {procs:?}"),
        })?;
    let xdim = procs
        .get("$XDIM")
        .ok_or(Error::NmrError {
            message: format!("$XDIM variable missing from {procs:?}"),
        })?
        .as_integer()
        .ok_or(Error::NmrError {
            message: format!("$XDIM variable is not an integer in {procs:?}"),
        })?;

    let endianness = match procs.get("$BYTEORD") {
        None => Endianness::Little,
        Some(byte_order) => {
            let byte_order = byte_order.as_integer().ok_or(Error::NmrError {
                message: format!("$BYTEORD variable is not an integer in {procs:?}"),
            })?;
            if byte_order == 1 {
                Endianness::Big
            } else {
                Endianness::Little
            }
        }
    };
    let data_type = match procs.get("$DTYPP") {
        None => DataType::Integer32,
        Some(dtype) => {
            let dtype = dtype.as_integer().ok_or(Error::NmrError {
                message: format!("$DTYPP variable is not an integer in {procs:?}"),
            })?;
            if dtype == 2 {
                DataType::Float64
            } else {
                DataType::Integer32
            }
        }
    };
    let mut data = read_bruker_binary_file(binary, data_type, endianness)?;
    scale_data(&mut data);
    Ok(())
}
