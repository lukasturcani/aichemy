use std::collections::HashMap;
use std::str;

use super::Error;

mod parser;
mod scanner;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Text(String),
    Number(f64),
    Array(Vec<f64>),
}

/// A parser for JCAMP-DX files.
///
/// This parser is based on the JCAMP-DX specification, defined
/// [here](http://www.jcamp-dx.org/protocols/dxir01.pdf),
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_NMR_1993.pdf) and
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)
/// TODO: stuff
#[derive(Clone, Debug, Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    pub fn parse(mut self, input: &str) -> Result<HashMap<String, Value>, Error> {
        let tokens = scanner::scan_tokens(input.as_bytes());
        todo!();
    }
}
