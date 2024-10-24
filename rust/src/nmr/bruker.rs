//! Tools to interact with Bruker data.
//!
//! # Examples
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use aichemy::nmr::bruker::{self, Procs};
//! use aichemy::nmr::io::jcamp_dx;
//! use std::fs;
//! use find_peaks::PeakFinder;
//! let procs = Procs(jcamp_dx::parse(fs::read("procs")?)?);
//! let mut spectrum =
//!     bruker::read_binary(fs::read("1r")?, procs.data_type()?, procs.endianness()?)?;
//! bruker::scale(&mut spectrum, procs.scale()?);
//! let peaks = PeakFinder::new(&spectrum)
//!     .with_min_prominence(1e4)
//!     .find_peaks();
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Error;

use super::io::jcamp_dx::Value;

/// The data type of the values in the spectrum binary file.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    /// 64-bit floating point values.
    Float64,
    /// 32-bit integer values.
    Integer32,
}

/// The endianness of the values in the spectrum binary file.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Endianness {
    /// Little endian.
    Little,
    /// Big endian.
    Big,
}

/// Interpret the content of a Bruker binary file spectrum.
///
/// # Examples
/// See [here](self#examples).
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

/// Scale the values in a spectrum.
///
/// Multiplies each value in the spectrum by the provided scale factor.
pub fn scale(data: &mut [f64], scale: f64) {
    for value in data {
        *value *= scale;
    }
}

/// A wrapper for `procs` files.
///
/// `procs` files are simple key-value files but some keys contain semantic
/// information that can be used to interpret the spectrum. This wrapper makes it
/// eassy to access this information. Typically, this structure will be used along
/// with functions like [`read_binary`].
///
/// # Examples
///
/// See [here](self#examples).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Procs(pub HashMap<String, Value>);

impl Procs {
    /// Return the endianness of the values in the spectrum binary file.
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

    /// Return the data type of the values in the spectrum binary file.
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

    /// Return the scaling factor for the values in the spectrum binary file.
    pub fn scale(&self) -> Result<f64, Error> {
        match self.0.get("$NCPROC") {
            None => Ok(1.0),
            Some(value) => {
                let value = match value {
                    Value::Float(value) => *value,
                    Value::Integer(value) => *value as f64,
                    _ => {
                        return Err(Error::NmrError {
                            message: format!("$NCPROC variable is not a number: {value:?}"),
                        })
                    }
                };
                Ok(2.0_f64.powf(value))
            }
        }
    }
}

pub fn peaks_df() {
    todo!()
}
