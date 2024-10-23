//! Tools for the JCAMP-DX format.
//!
//! More details about the JCAMP-DX format can be found
//! [here](http://www.jcamp-dx.org/protocols/dxir01.pdf),
//! [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_NMR_1993.pdf) and
//! [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)

use super::Error;

mod parser;
mod scanner;

pub use parser::{parse, Value};
