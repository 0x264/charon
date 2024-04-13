use std::str::FromStr;
use crate::token::{Token, TokenKind};

fn identifier_or_keyword(s: String) -> TokenKind {
    match s.as_str() {
        "var" => TokenKind::Var,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "if" => TokenKind::If,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "return" => TokenKind::Return,
        "func" => TokenKind::Func,
        "class" => TokenKind::Class,
        "this" => TokenKind::This,
        "null" => TokenKind::Null,
        _ => TokenKind::Identifier(s)
    }
}

pub struct Error {
    msg: String,
    offset: usize
}

impl Error {
    fn new(msg: String, offset: usize) -> Self {
        Self {
            msg,
            offset
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub struct Lexer<'a> {
    data: &'a [u8],
    offset: usize,
    buf: String
}

impl Lexer<'_> {
    pub fn new(data: &[u8]) -> Lexer {
        Lexer {
            data,
            offset: 0,
            buf: String::new()
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            let off = self.offset;
            self.advance();

            let c = c as char;
            let tok = match c {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ';' => TokenKind::Semi,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '=' => if self.consume('=') {
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
                '>' => if self.consume('=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
                '<' => if self.consume('=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
                '!' => if self.consume('=') {
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
                '&' => if self.consume('&') {
                    TokenKind::AmpAmp
                } else {
                    return Err(Error::new("not support single &".to_owned(), off));
                }
                '|' => if self.consume('|') {
                    TokenKind::BarBar
                } else {
                    return Err(Error::new("not support single |".to_owned(), off));
                }
                '+' => if self.consume('=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
                '-' => if self.consume('=') {
                    TokenKind::SubEq
                } else {
                    TokenKind::Sub
                }
                '*' => if self.consume('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
                '/' => if self.consume('/') {
                    while let Some(v) = self.next() {
                        if v == b'\n' {
                            break;
                        }
                    }
                    continue;
                } else if self.consume('=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
                ' ' | '\t' | '\n' | '\r' => continue,
                _ => if c == '"' {
                    self.parse_string_literal()?
                } else if c.is_ascii_digit() {
                    match self.parse_long_double(c) {
                        Ok(t) => t,
                        Err(e) => return Err(Error::new(e, off))
                    }
                } else if matches!(c, 'a'..='z' | 'A'..='Z' | '_') {
                    self.parse_identifier_keyword(c)
                } else {
                    return Err(Error::new(format!("unsupport char: {c}"), off));
                }
            };
            tokens.push(Token::new(tok, off));
        }

        Ok(tokens)
    }

    fn parse_string_literal(&mut self) -> Result<TokenKind, Error> {
        let mut s = String::new();

        let off = self.offset;
        while let Some(c) = self.next() {
            let c = c as char;
            match c {
                '\\' => if let Some(v) = self.next() {
                    let v = v as char;
                    let t = match v {
                        '\\' => '\\',
                        'r' => '\r',
                        'n' => '\n',
                        '"' => '"',
                        't' => '\t',
                        _ => return Err(Error::new(format!("unsupport escape sequence: \\{v}"), self.offset - 2))
                    };
                    s.push(t);
                } else {
                    return Err(Error::new("escape sequence: no char found after: \\".to_owned(), self.offset - 1));
                }
                '"' => return Ok(TokenKind::String(s)),
                _ => s.push(c)
            }
        }

        Err(Error::new("unclosed string literal".to_owned(), off - 1))
    }

    fn parse_long_double(&mut self, first: char) -> Result<TokenKind, String> {
        self.buf.clear();
        self.buf.push(first);

        let mut has_dot = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
                self.buf.push(c as char);
            } else if !has_dot && c == b'.' {
                self.advance();
                self.buf.push(c as char);
                has_dot = true;
            } else {
                break;
            }
        }

        if has_dot {
            match f64::from_str(&self.buf) {
                Ok(v) => Ok(TokenKind::Double(v)),
                Err(e) => Err(e.to_string())
            }
        } else {
            match i64::from_str(&self.buf) {
                Ok(v) => Ok(TokenKind::Long(v)),
                Err(e) => Err(e.to_string())
            }
        }
    }

    fn parse_identifier_keyword(&mut self, first: char) -> TokenKind {
        let mut s = String::new();
        s.push(first);

        while let Some(c) = self.peek() && c.is_ascii_alphanumeric() {
            self.advance();
            s.push(c as char);
        }

        identifier_or_keyword(s)
    }

    fn next(&mut self) -> Option<u8> {
        self.data.get(self.offset).map(|v| {
            self.advance();
            *v
        })
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.offset).copied()
    }

    fn advance(&mut self) {
        self.offset += 1;
    }

    fn consume(&mut self, c: char) -> bool {
        if let Some(v) = self.peek() && v as char == c {
            self.advance();
            true
        } else {
            false
        }
    }
}
