use std::{collections::HashMap, mem, str};

use serde::{Deserialize, Serialize};

use crate::nmr::io::Error;

use super::scanner::{scan_tokens, Token, TokenType};

/// A JCAMP-DX value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// A string.
    String(String),
    /// A number.
    Number(f64),
    /// An array of numbers.
    NumberArray(Vec<f64>),
    /// An array of strings.
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEndOfFile,
}

fn error_token(token: &TokenType) -> String {
    match token {
        TokenType::DataLabel(label) => format!("data label: {}", label),
        TokenType::String(string) => format!("string: {:?}", string),
        TokenType::Number(number) => format!("number: {}", number),
        TokenType::Int(number) => format!("integer: {}", number),
        TokenType::BeginVariableList(list) => format!("variable list: {}", list),
        TokenType::OpenBracket => "open bracket".into(),
        TokenType::CloseBracket => "close bracket".into(),
        TokenType::DoubleDot => "double dot".into(),
        TokenType::NewLine => "new line".into(),
    }
}

fn error_line(lines: &Option<Vec<&str>>, error: &ParseError) -> String {
    match error {
        ParseError::UnexpectedToken(token) => {
            format!(
                "unexpected token on line {}: [{}] {}",
                token.line,
                lines.as_ref().map_or("", |lines| lines[token.line - 1]),
                error_token(&token.r#type)
            )
        }
        ParseError::UnexpectedEndOfFile => "Unexpected end of file".into(),
    }
}

fn error_msg(source: &[u8], error: Vec<ParseError>) -> String {
    let lines = str::from_utf8(source)
        .ok()
        .map(|s| s.lines().collect::<Vec<_>>());
    error
        .iter()
        .map(|error| error_line(&lines, error))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Parser JCAMP-DX files.
///
/// This parser is based on the JCAMP-DX specification, defined
/// [here](http://www.jcamp-dx.org/protocols/dxir01.pdf),
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_NMR_1993.pdf) and
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)
pub fn parse(source: impl AsRef<[u8]>) -> Result<HashMap<String, Value>, Error> {
    let source = source.as_ref();
    Parser::new(scan_tokens(source)?)
        .parse()
        .map_err(|error| Error::Parse(error_msg(source, error)))
}

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
            _ => Err(ParseError::UnexpectedToken(
                self.tokens[self.current].clone(),
            )),
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
                Ok(Value::NumberArray(self.variable_list()?))
            }
            TokenType::OpenBracket => self.array(),
            _ => Err(ParseError::UnexpectedToken(
                self.tokens[self.current].clone(),
            )),
        };
        self.consume_newlines();
        result
    }

    fn consume_type(&mut self, expected: &TokenType) -> Result<Token, ParseError> {
        let result = match self.tokens.get(self.current) {
            Some(token) if mem::discriminant(&token.r#type) == mem::discriminant(expected) => {
                Ok(token.clone())
            }
            Some(token) => Err(ParseError::UnexpectedToken(token.clone())),
            None => Err(ParseError::UnexpectedEndOfFile),
        };
        self.current += 1;
        result
    }

    fn array(&mut self) -> Result<Value, ParseError> {
        self.consume_type(&TokenType::OpenBracket)?;
        self.consume_type(&TokenType::Int(0))?;
        self.consume_type(&TokenType::DoubleDot)?;

        match self.tokens.get(self.current + 3) {
            Some(token) => match &token.r#type {
                TokenType::Number(_) => self.number_array(),
                TokenType::String(_) => self.string_array(),
                _ => Err(ParseError::UnexpectedToken(token.clone())),
            },
            None => Err(ParseError::UnexpectedEndOfFile),
        }
    }

    fn number_array(&mut self) -> Result<Value, ParseError> {
        let mut array = Vec::new();
        if let TokenType::Int(max_index) = self.consume_type(&TokenType::Int(0))?.r#type {
            array.reserve(max_index + 1);
        };
        self.consume_type(&TokenType::CloseBracket)?;
        while let Some(token) = self.tokens.get(self.current) {
            match token.r#type {
                TokenType::Number(number) => {
                    array.push(number);
                    self.current += 1;
                }
                TokenType::NewLine => self.current += 1,
                TokenType::DataLabel(_) => break,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.tokens[self.current].clone(),
                    ));
                }
            }
        }
        Ok(Value::NumberArray(array))
    }

    fn string_array(&mut self) -> Result<Value, ParseError> {
        let mut array = Vec::new();
        if let TokenType::Int(max_index) = self.consume_type(&TokenType::Int(0))?.r#type {
            array.reserve(max_index + 1);
        };
        self.consume_type(&TokenType::CloseBracket)?;
        while let Some(token) = self.tokens.get(self.current) {
            match &token.r#type {
                TokenType::String(string) => {
                    array.push(string.clone());
                    self.current += 1;
                }
                TokenType::NewLine => self.current += 1,
                TokenType::DataLabel(_) => break,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.tokens[self.current].clone(),
                    ));
                }
            }
        }
        Ok(Value::StringArray(array))
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
                        self.tokens[self.current].clone(),
                    ))
                }
            }
        }
        Ok(array)
    }
}
