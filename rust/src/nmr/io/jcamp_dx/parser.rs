use std::collections::HashMap;

use super::scanner::{Token, TokenType};

pub enum Value {
    String(String),
    Number(f64),
    Array(Vec<f64>),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<HashMap<String, Value>, ParseError> {
        let mut map = HashMap::new();
        while self.current < self.tokens.len() {
            let (key, value) = self.record()?;
            map.insert(key, value);
        }
        Ok(map)
    }

    fn record(&mut self) -> Result<(String, Value), ParseError> {
        Ok((self.data_label()?, self.data_set()?))
    }

    fn data_label(&mut self) -> Result<String, ParseError> {
        match &self.tokens[self.current].r#type {
            TokenType::DataLabel(data_label) => {
                self.current += 1;
                Ok(data_label.clone())
            }
            _ => Err(ParseError::UnexpectedToken("expected data label".into())),
        }
    }

    fn data_set(&mut self) -> Result<Value, ParseError> {
        match &self.tokens[self.current].r#type {
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
                Ok(Value::Array(self.variable_list()?))
            }
            _ => Err(ParseError::UnexpectedToken("expected value".into())),
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
