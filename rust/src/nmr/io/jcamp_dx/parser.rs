use std::collections::HashMap;

use crate::nmr::io::Error;

use super::scanner::{scan_tokens, Token, TokenType};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Array(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    UnexpectedToken(String),
}

pub fn parse(source: &[u8]) -> Result<HashMap<String, Value>, Error> {
    Parser::new(scan_tokens(source)?)
        .parse()
        .map_err(|error| Error::Parse(format!("{:?}", error)))
}

/// A parser for JCAMP-DX files.
///
/// This parser is based on the JCAMP-DX specification, defined
/// [here](http://www.jcamp-dx.org/protocols/dxir01.pdf),
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_NMR_1993.pdf) and
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)
/// TODO: stuff
#[derive(Clone, Debug, Default)]
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<HashMap<String, Value>, Vec<ParseError>> {
        let mut map = HashMap::new();
        let mut errors = Vec::new();
        while self.current < self.tokens.len() {
            match self.record() {
                Ok((key, value)) => {
                    map.insert(key, value);
                }
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            }
        }
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(map)
        }
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.tokens.get(self.current) {
            if let TokenType::DataLabel(_) = token.r#type {
                break;
            }
            self.current += 1;
        }
    }

    fn record(&mut self) -> Result<(String, Value), ParseError> {
        let data_label = self.data_label()?;
        self.consume_newlines();
        let data_set = match self.tokens.get(self.current) {
            Some(token) => match token.r#type {
                TokenType::DataLabel(_) => Value::String("".into()),
                _ => self.data_set()?,
            },
            None => Value::String("".into()),
        };
        Ok((data_label, data_set))
    }

    fn data_label(&mut self) -> Result<String, ParseError> {
        match &self.tokens[self.current].r#type {
            TokenType::DataLabel(data_label) => {
                self.current += 1;
                Ok(data_label.clone())
            }
            _ => Err(ParseError::UnexpectedToken(format!(
                "{:?}: expected data label",
                self.tokens[self.current]
            ))),
        }
    }

    fn data_set(&mut self) -> Result<Value, ParseError> {
        let result = match &self.tokens[self.current].r#type {
            TokenType::String(string) => {
                self.current += 1;
                Ok(Value::String(string.clone()))
            }
            &TokenType::Number(number) => {
                self.current += 1;
                Ok(Value::Number(number))
            }
            TokenType::BeginVariableList(_) => {
                self.current += 1;
                self.consume_newlines();
                Ok(Value::Array(self.variable_list()?))
            }
            _ => Err(ParseError::UnexpectedToken("expected value".into())),
        };
        self.consume_newlines();
        result
    }

    fn consume_newlines(&mut self) {
        while let Some(token) = self.tokens.get(self.current) {
            if token.r#type == TokenType::NewLine {
                self.current += 1;
            } else {
                break;
            }
        }
    }

    fn variable_list(&mut self) -> Result<Vec<f64>, ParseError> {
        let mut take_number = false;
        let mut array = Vec::new();
        while let Some(token) = self.tokens.get(self.current) {
            match token.r#type {
                TokenType::Number(number) if take_number => {
                    self.current += 1;
                    array.push(number);
                }
                TokenType::Number(_) => {
                    self.current += 1;
                    take_number = true;
                }
                TokenType::NewLine => {
                    self.current += 1;
                    take_number = false;
                }
                TokenType::DataLabel(_) => break,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected number or data label".into(),
                    ))
                }
            }
        }
        Ok(array)
    }
}
