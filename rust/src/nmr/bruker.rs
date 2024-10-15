//! Tools to interact with Bruker data.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use thiserror::Error;

/// Error which may occur when interacting with Bruker data.
#[derive(Error, Debug)]
pub enum Error {
    /// If the file cannot be processed for some reason.
    #[error("Could not process file: {source}")]
    InvalidFile { source: std::io::Error },
    /// Invalid Data
    #[error("File contains invalid data: {0}")]
    InvalidData(String),
}

pub fn read_pdata(dir: impl AsRef<Path>) -> Result<(), Error> {
    let dir = dir.as_ref();
    Ok(())
}

enum JcampValue {
    Bool(bool),
    Vec(Vec<i64>),
    String(String),
    Float(f64),
    Int(i64),
    None,
}

impl FromStr for JcampValue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(Self::Bool(true)),
            "no" => Ok(Self::Bool(false)),
            "" => Ok(Self::None),
            _ if s.starts_with('<') && s.ends_with('>') => {
                Ok(Self::String(s[1..s.len() - 1].into()))
            }
            _ => Ok(Self::String(s.into())),
        }
    }
}

struct JcampData {
    core_header: Vec<String>,
    comments: Vec<String>,
    keys: HashMap<String, JcampValue>,
}

fn read_jcamp(path: impl AsRef<Path>) -> Result<JcampData, Error> {
    let path = path.as_ref();
    let mut core_header = vec![];
    let mut comments = vec![];
    let content = fs::read_to_string(path).map_err(|source| Error::InvalidFile { source })?;

    let mut keys = HashMap::new();
    for line in JcampLines::new(&content) {
        if line.starts_with("$$") {
            comments.push(line.into());
        } else if let Some(line) = line.strip_prefix("##$") {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| Error::InvalidData(format!("Invalid line: {}", line)))?;
            keys.insert(key.into(), value.trim().parse::<JcampValue>()?);
        } else if line.starts_with("##") {
            core_header.push(line.into());
        }
    }
    Ok(JcampData {
        core_header,
        comments,
        keys,
    })
}

struct JcampLines<'s> {
    content: &'s str,
    current_index: usize,
}

impl<'s> JcampLines<'s> {
    fn new(content: &'s str) -> Self {
        Self {
            content,
            current_index: 0,
        }
    }
}

impl<'s> Iterator for JcampLines<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.content.len() {
            return None;
        }
        let mut ignore_newline = false;
        for (index, char) in self.content[self.current_index..]
            .char_indices()
            .map(|(i, c)| (i + self.current_index, c))
        {
            match char {
                '<' => {
                    ignore_newline = true;
                }
                '>' => {
                    ignore_newline = false;
                }
                '\n' if !ignore_newline => {
                    let current_index = self.current_index;
                    self.current_index = index + 1;
                    return Some(&self.content[current_index..index]);
                }
                _ => continue,
            }
        }
        let current_index = self.current_index;
        self.current_index = self.content.len();
        Some(&self.content[current_index..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jcamp_lines() {
        let content = r#"
##TITLE= Parameter file, TOPSPIN		Version 3.2
##JCAMPDX= 5.0
##DATATYPE= Parameter Values
##NPOINTS= 9	$$ modification sequence number
##ORIGIN= Bruker BioSpin GmbH
##OWNER= nmrsu
$$ 2022-07-21 09:31:58.111 +0100  nmrsu@nmrpc
$$ /opt/topspin/data/bg/nmr/Jul20-2022/1920/pdata/1/proc
$$ process /opt/topspin3.2/prog/au/bin/user/proc_1dIC3.2
##$ABSF1= 10
##$AUNMP= <proc_1dIC3.2>
##$AXNUC= <off>
##$AXUNIT= <>
##$AZFW= 0.1
##$ERETIC= no
##$F2P= -1
##$PHC0= -61.86225
##$PKNL= yes
##$PPMPNUM= 2147483647
##$PYNMP= <proc.py>
##$SREGLST= <1H.CDCl3>
##$TI= <AB-01-005 P4-48

>
##$USERP3= <>
##$USERP4= <>
##$USERP5= <>
##$WDW= 1
##$XDIM= 0
##$YMAX_p= 0
##$YMIN_p= 0
##END="#;
    }
}
