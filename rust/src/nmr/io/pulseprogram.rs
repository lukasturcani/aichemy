//! Tools for bruker pulseprogram files.
//!
//! More details about the pulseprogram format can be found
//! [here](http://www2.chem.uic.edu/nmr/downloads/pulse.pdf).

use std::collections::HashMap;

use super::Error;

pub enum Value {}

pub struct PulseprogramContent {
    variables: HashMap<String, Value>,
    increment_times: Vec<f64>,
}

/// Parse a pulseprogram file.
///
/// More details about the pulseprogram format can be found
/// [here](http://www2.chem.uic.edu/nmr/downloads/pulse.pdf).
pub fn parse(source: impl AsRef<str>) -> Result<PulseprogramContent, Error> {
    let source = source.as_ref();
    for line in source
        .lines()
        // remove comments from lines
        .map(|line| line.split_once(';').map_or(line, |(valid, _)| valid))
        // remove empty lines
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        // remove includes
        .filter(|line| !line.starts_with('#'))
    {
        if line.starts_with('"') {
            line.split_once('=')
        }
    }

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
