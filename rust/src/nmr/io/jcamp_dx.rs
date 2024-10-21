use std::collections::HashMap;
use std::str;

use super::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Text(String),
    Number(f64),
    Array(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    DataLabel(String),
    String(String),
    Number(f64),
    BeginVariableList(String),
    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    line: usize,
    r#type: TokenType,
}

#[derive(Debug, Clone)]
enum ScanError {
    UnexpectedCharacter { line: usize, character: char },
    InvalidString { line: usize },
    ExpectedNumber { line: usize },
}

struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<ScanError>,
}

impl Scanner {
    fn new() -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn scan_tokens(mut self, source: &[u8]) -> Result<Vec<Token>, Error> {
        while let Some(&char) = source.get(self.current) {
            match char {
                b'$' => {
                    if self.r#match(source, b'$') {
                        self.handle_inline_comment(source);
                    } else {
                        self.add_error(ScanError::UnexpectedCharacter {
                            line: self.line,
                            character: char as char,
                        });
                    }
                }
                b'#' => {
                    if self.r#match(source, b'#') {
                        if self.r#match(source, b'=') {
                            self.handle_multiline_comment(source)
                        } else {
                            self.handle_data_label(source);
                            self.handle_data_set(source);
                        }
                    } else {
                        self.add_error(ScanError::UnexpectedCharacter {
                            line: self.line,
                            character: char as char,
                        });
                    }
                }
                b' ' | b'\t' | b'\r' => self.advance(),
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => self.add_error(ScanError::UnexpectedCharacter {
                    line: self.line,
                    character: char as char,
                }),
            }
        }
        if !self.errors.is_empty() {
            Err(Error::Parse(format!("{:?}", self.errors)))
        } else {
            Ok(self.tokens)
        }
    }

    fn is_number_start(char: u8) -> bool {
        char.is_ascii_digit() || char == b'+' || char == b'-' || char == b'.'
    }

    fn handle_data_set(&mut self, source: &[u8]) {
        while let Some(&char) = source.get(self.current) {
            match char {
                b'$' => {
                    if self.r#match(source, b'$') {
                        self.handle_inline_comment(source);
                    } else {
                        self.current += 1;
                    }
                }
                b'\n' => {
                    self.add_token(TokenType::NewLine);
                    self.line += 1;
                }
                b'#' => {
                    if source.get(self.current + 1) == Some(&b'#') {
                        break;
                    } else {
                        self.current += 1;
                    }
                }
                _ if Scanner::is_number_start(char) => {
                    while let Some(next) = source.get(self.current + 1) {
                        if next.is_ascii_whitespace() {
                            break;
                        }
                        self.current += 1;
                    }
                    match str::from_utf8(&source[self.start..self.current + 1]) {
                        Ok(string) => match string.parse::<f64>() {
                            Ok(number) => self.add_token(TokenType::Number(number)),
                            Err(_) => self.add_error(ScanError::ExpectedNumber { line: self.line }),
                        },
                        Err(_) => self.add_error(ScanError::InvalidString { line: self.line }),
                    }
                }
                _ if char.is_ascii_graphic() => {
                    while let Some(&next) = source.get(self.current + 1) {
                        if (next == b'$' && source.get(self.current + 2) == Some(&b'$'))
                            || next == b'\n'
                        {
                            break;
                        }
                        self.current += 1;
                    }
                    match str::from_utf8(source[self.start..self.current + 1].trim_ascii_end()) {
                        Ok(string) => self.add_token(TokenType::String(string.into())),
                        Err(_) => self.add_error(ScanError::InvalidString { line: self.line }),
                    }
                }
                _ => self.advance(),
            }
        }
    }

    fn handle_inline_comment(&mut self, source: &[u8]) {
        while let Some(&char) = source.get(self.current + 1) {
            if char == b'\n' {
                break;
            }
            self.current += 1;
        }
        self.advance();
    }

    fn handle_multiline_comment(&mut self, source: &[u8]) {
        while let Some(&char) = source.get(self.current) {
            match char {
                b'\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                b'#' => {
                    if source.get(self.current + 1) == Some(&b'#') {
                        self.start = self.current;
                        break;
                    }
                }
                _ => {
                    self.current += 1;
                }
            }
        }
    }

    fn handle_data_label(&mut self, source: &[u8]) {
        let mut identifier = String::new();
        while let Some(&current) = source.get(self.current) {
            if current == b'=' {
                break;
            }
            if current.is_ascii_alphanumeric() {
                identifier.push(current.to_ascii_uppercase() as char)
            }
            self.current += 1;
        }
        self.add_token(TokenType::DataLabel(identifier));
    }

    fn r#match(&mut self, source: &[u8], char: u8) -> bool {
        if let Some(&next) = source.get(self.current + 1) {
            if next == char {
                self.current += 1;
                return true;
            }
        }
        false
    }

    fn advance(&mut self) {
        self.current += 1;
        self.start = self.current;
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.tokens.push(Token {
            line: self.line,
            r#type,
        });
        self.advance()
    }

    fn add_error(&mut self, error: ScanError) {
        self.errors.push(error);
        self.advance();
    }
}

fn scan_tokens(source: &[u8]) -> Result<Vec<Token>, Error> {
    Scanner::new().scan_tokens(source)
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
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_data_label() {
        let tokens = scan_tokens(b"##.mY/d atalabEl =").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    line: 1,
                    r#type: TokenType::DataLabel("MYDATALABEL".into())
                },
                Token {
                    line: 1,
                    r#type: TokenType::String("".into())
                }
            ]
        );
    }

    #[test]
    fn scan_multiline_comment() {
        let tokens = scan_tokens(
            b"
                ##label 1= foo
                ##= this is a comment anything
                can be put here
                ##END= bar
            ",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    line: 2,
                    r#type: TokenType::DataLabel("LABEL1".into())
                },
                Token {
                    line: 2,
                    r#type: TokenType::String("foo".into()),
                },
                Token {
                    line: 2,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 5,
                    r#type: TokenType::DataLabel("END".into())
                },
                Token {
                    line: 5,
                    r#type: TokenType::String("bar".into()),
                },
                Token {
                    line: 5,
                    r#type: TokenType::NewLine
                },
            ]
        );
    }

    #[test]
    fn scan_inline_comment() {
        let tokens = scan_tokens(b"  $$ this is a comment ").unwrap();
        assert_eq!(tokens, vec![]);

        let tokens = scan_tokens(b" ##foo=  $$ this is a comment ").unwrap();
        assert_eq!(
            tokens,
            vec![Token {
                line: 1,
                r#type: TokenType::DataLabel("FOO".into())
            },]
        );
    }

    #[test]
    fn scan_string_record() {
        let tokens = scan_tokens(
            b"
                ##label 1 =  this is a string  \n\
                ##label 2 = also this $$ ignore me
                ##label 3 = and this
                ##label 4 =
                ##label 5 = foo
            ",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    line: 2,
                    r#type: TokenType::DataLabel("LABEL1".into())
                },
                Token {
                    line: 2,
                    r#type: TokenType::String("this is a string".into())
                },
                Token {
                    line: 2,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 3,
                    r#type: TokenType::DataLabel("LABEL2".into())
                },
                Token {
                    line: 3,
                    r#type: TokenType::String("also this".into())
                },
                Token {
                    line: 3,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 4,
                    r#type: TokenType::DataLabel("LABEL3".into())
                },
                Token {
                    line: 4,
                    r#type: TokenType::String("and this".into())
                },
                Token {
                    line: 4,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 5,
                    r#type: TokenType::DataLabel("LABEL4".into())
                },
                Token {
                    line: 5,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 6,
                    r#type: TokenType::DataLabel("LABEL5".into())
                },
                Token {
                    line: 6,
                    r#type: TokenType::String("foo".into())
                },
                Token {
                    line: 6,
                    r#type: TokenType::NewLine
                }
            ]
        );
    }

    #[test]
    fn scan_number_record() {
        let tokens = scan_tokens(
            b"
                ##label 1 =   .32  \n\
                ##label 2 = -42.32 $$ ignore me
                ##label 3 = 42
                ##label 4 = 42e12
            ",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    line: 2,
                    r#type: TokenType::DataLabel("LABEL1".into())
                },
                Token {
                    line: 3,
                    r#type: TokenType::Number(0.32)
                },
                Token {
                    line: 3,
                    r#type: TokenType::DataLabel("LABEL2".into())
                },
                Token {
                    line: 3,
                    r#type: TokenType::Number(-42.32)
                },
                Token {
                    line: 4,
                    r#type: TokenType::DataLabel("LABEL3".into())
                },
                Token {
                    line: 5,
                    r#type: TokenType::Number(42.),
                },
                Token {
                    line: 5,
                    r#type: TokenType::DataLabel("LABEL4".into())
                },
                Token {
                    line: 6,
                    r#type: TokenType::Number(42e12),
                },
            ]
        );
    }

    #[test]
    fn scan_asdf_data_set() {
        let tokens = scan_tokens(
            b"
                ##label 1 =  (X++(Y..Y))
              123 0.53 0.43
              456 0.32 0.22
            ",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    line: 2,
                    r#type: TokenType::DataLabel("LABEL1".into())
                },
                Token {
                    line: 2,
                    r#type: TokenType::BeginVariableList("X++(Y..Y)".into())
                },
                Token {
                    line: 2,
                    r#type: TokenType::NewLine,
                },
                Token {
                    line: 3,
                    r#type: TokenType::Number(123.)
                },
                Token {
                    line: 3,
                    r#type: TokenType::Number(0.53)
                },
                Token {
                    line: 3,
                    r#type: TokenType::Number(0.43)
                },
                Token {
                    line: 4,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 5,
                    r#type: TokenType::Number(456.)
                },
                Token {
                    line: 5,
                    r#type: TokenType::Number(0.32)
                },
                Token {
                    line: 5,
                    r#type: TokenType::Number(0.22)
                },
                Token {
                    line: 6,
                    r#type: TokenType::NewLine
                }
            ]
        );
    }
}
