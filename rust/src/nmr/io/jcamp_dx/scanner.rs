use super::Error;
use std::str;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub line: usize,
    pub r#type: TokenType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    DataLabel(String),
    String(String),
    Number(f64),
    Int(usize),
    BeginVariableList(String),
    OpenBracket,
    CloseBracket,
    DoubleDot,
    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
enum ScanError {
    UnexpectedCharacter { line: usize, character: char },
    InvalidString { line: usize },
    ExpectedNumber { line: usize },
    ExpectedInt { line: usize },
    UnterminatedString { line: usize },
    ExpectedDot { line: usize },
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
                            self.current += 1;
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
                b'<' => self.handle_multiline_string(source),
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
                b'(' => {
                    if let Some(next) = source.get(self.current + 1) {
                        if next.is_ascii_digit() {
                            self.handle_array_prefix(source);
                        } else {
                            self.handle_string(source);
                        }
                    }
                }
                _ if Scanner::is_number_start(char) => self.handle_number(source),
                _ if char.is_ascii_graphic() => self.handle_string(source),
                _ => self.advance(),
            }
        }
    }

    fn handle_number(&mut self, source: &[u8]) {
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

    fn handle_int(&mut self, source: &[u8]) {
        while let Some(next) = source.get(self.current + 1) {
            if next.is_ascii_whitespace() {
                break;
            }
            self.current += 1;
        }
        match str::from_utf8(&source[self.start..self.current + 1]) {
            Ok(string) => match string.parse::<usize>() {
                Ok(number) => self.add_token(TokenType::Int(number)),
                Err(_) => self.add_error(ScanError::ExpectedInt { line: self.line }),
            },
            Err(_) => self.add_error(ScanError::InvalidString { line: self.line }),
        }
    }

    fn handle_array_prefix(&mut self, source: &[u8]) {
        while let Some(token) = source.get(self.current) {
            match token {
                b'(' => self.add_token(TokenType::OpenBracket),
                b')' => {
                    self.add_token(TokenType::CloseBracket);
                    break;
                }
                b'.' => {
                    if self.r#match(source, b'.') {
                        self.add_token(TokenType::DoubleDot);
                    } else {
                        self.add_error(ScanError::ExpectedDot { line: self.line });
                    }
                }
                _ => self.handle_int(source),
            }
        }
    }

    fn handle_string(&mut self, source: &[u8]) {
        while let Some(&next) = source.get(self.current + 1) {
            if (next == b'$' && source.get(self.current + 2) == Some(&b'$')) || next == b'\n' {
                break;
            }
            self.current += 1;
        }
        match str::from_utf8(source[self.start..self.current + 1].trim_ascii_end()) {
            Ok(string) => {
                if string == "(X++(Y..Y))" {
                    self.add_token(TokenType::BeginVariableList("(X++(Y..Y))".into()));
                } else {
                    self.add_token(TokenType::String(string.into()));
                }
            }
            Err(_) => self.add_error(ScanError::InvalidString { line: self.line }),
        }
    }

    fn handle_multiline_string(&mut self, source: &[u8]) {
        while let Some(&char) = source.get(self.current) {
            match char {
                b'\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                b'>' => {
                    break;
                }
                _ => self.current += 1,
            }
        }
        if source.get(self.current).is_none() {
            self.add_error(ScanError::UnterminatedString { line: self.line });
        } else {
            match str::from_utf8(&source[self.start + 1..self.current]) {
                Ok(string) => self.add_token(TokenType::String(string.into())),
                Err(_) => self.add_error(ScanError::InvalidString { line: self.line }),
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
        match source.get(self.current) {
            Some(b'.') => {
                identifier.push('.');
                self.current += 1;
            }
            Some(b'$') => {
                identifier.push('$');
                self.current += 1;
            }
            _ => {}
        }
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

pub fn scan_tokens(source: &[u8]) -> Result<Vec<Token>, Error> {
    Scanner::new().scan_tokens(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_data_label() {
        let tokens = scan_tokens(b"##.mY/d atalabEl =").unwrap();
        assert_eq!(
            tokens,
            vec![Token {
                line: 1,
                r#type: TokenType::DataLabel(".MYDATALABEL".into())
            },]
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
    fn scan_multiline_string_record() {
        let tokens = scan_tokens(
            b"
                ##label 1 =  <this is a string>  \n\
                ##label 2 = <also this\n  foo>
                ##label 3 = <>
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
                    line: 4,
                    r#type: TokenType::String("also this\n  foo".into())
                },
                Token {
                    line: 4,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 5,
                    r#type: TokenType::DataLabel("LABEL3".into())
                },
                Token {
                    line: 5,
                    r#type: TokenType::String("".into())
                },
                Token {
                    line: 5,
                    r#type: TokenType::NewLine
                },
            ]
        );

        let tokens = scan_tokens(
            b"
                ##label 1 =  <this is a string
                ##label 2 =  foo
            ",
        );
        assert_eq!(
            tokens,
            Err(Error::Parse(format!(
                "{:?}",
                vec![ScanError::UnterminatedString { line: 4 }]
            )))
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
                    line: 2,
                    r#type: TokenType::Number(0.32)
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
                    r#type: TokenType::Number(-42.32)
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
                    r#type: TokenType::Number(42.),
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
                    r#type: TokenType::Number(42e12),
                },
                Token {
                    line: 5,
                    r#type: TokenType::NewLine
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
                    r#type: TokenType::BeginVariableList("(X++(Y..Y))".into())
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
                    line: 3,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(456.)
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(0.32)
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(0.22)
                },
                Token {
                    line: 4,
                    r#type: TokenType::NewLine
                }
            ]
        );
    }

    #[test]
    fn scan_array_data_set() {
        let tokens = scan_tokens(
            b"
                ##label 1 =  (0..5)
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
                    r#type: TokenType::OpenBracket,
                },
                Token {
                    line: 2,
                    r#type: TokenType::Int(0),
                },
                Token {
                    line: 2,
                    r#type: TokenType::DoubleDot,
                },
                Token {
                    line: 2,
                    r#type: TokenType::Int(5),
                },
                Token {
                    line: 2,
                    r#type: TokenType::CloseBracket,
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
                    line: 3,
                    r#type: TokenType::NewLine
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(456.)
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(0.32)
                },
                Token {
                    line: 4,
                    r#type: TokenType::Number(0.22)
                },
                Token {
                    line: 4,
                    r#type: TokenType::NewLine
                }
            ]
        );
    }
}
