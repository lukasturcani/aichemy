//! Tools to interact with Bruker data.

use std::{collections::HashMap, fs, path::Path};

use crate::Error;

use super::io::jcamp_dx::Value;

pub enum DataType {
    Float64,
    Integer32,
}

pub enum Endianness {
    Little,
    Big,
}

/// Interpret the content of a Bruker binary file.
///
/// # Examples
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use aichemy::nmr::bruker{self, Procs};
/// use aichemy::nmr::io::jcamp_dx;
/// use std::fs;
/// let procs = Procs(jcamp_dx::parse(fs::read("procs")?)?);
/// let spectrum = bruker::read_binary(fs::read("1r")?, procs.data_type()?, procs.endianness()?)?;
/// ```
pub fn read_binary(
    array: Vec<u8>,
    data_type: DataType,
    endianness: Endianness,
) -> Result<Vec<f64>, Error> {
    match (data_type, endianness) {
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

fn read_2d_spectrum(binary: impl AsRef<Path>, procs: impl AsRef<Path>, acqus: impl AsRef<Path>) {
    // let si = procs
    //     .get("$SI")
    //     .ok_or(Error::NmrError {
    //         message: format!("$SI variable missing from {procs:?}"),
    //     })?
    //     .as_integer()
    //     .ok_or(Error::NmrError {
    //         message: format!("$SI variable is not an integer in {procs:?}"),
    //     })?;
    // let xdim = procs
    //     .get("$XDIM")
    //     .ok_or(Error::NmrError {
    //         message: format!("$XDIM variable missing from {procs:?}"),
    //     })?
    //     .as_integer()
    //     .ok_or(Error::NmrError {
    //         message: format!("$XDIM variable is not an integer in {procs:?}"),
    //     })?;
    todo!();
}

pub struct Procs(pub HashMap<String, Value>);

impl Procs {
    pub fn endianness(&self) -> Result<Endianness, Error> {
        match self.0.get("$BYTEORD") {
            None => Ok(Endianness::Little),
            Some(byte_order) => {
                let byte_order = byte_order.as_integer().ok_or(Error::NmrError {
                    message: format!("$BYTEORD variable is not an integer: {byte_order:?}"),
                })?;
                if byte_order == 1 {
                    Ok(Endianness::Big)
                } else {
                    Ok(Endianness::Little)
                }
            }
        }
    }

    pub fn data_type(&self) -> Result<DataType, Error> {
        let dtype = self.0.get("$DTYPP");
        Ok(match dtype {
            None => DataType::Integer32,
            Some(dtype) => {
                let dtype = dtype.as_integer().ok_or(Error::NmrError {
                    message: format!("$DTYPP variable is not an integer: {dtype:?}"),
                })?;
                if dtype == 2 {
                    DataType::Float64
                } else {
                    DataType::Integer32
                }
            }
        })
    }
}
