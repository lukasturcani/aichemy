use std::collections::HashMap;
use std::str;

use super::Error;

mod parser;
mod scanner;

pub use parser::{parse, Value};
