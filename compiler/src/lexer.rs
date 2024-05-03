use std::str::FromStr;
use crate::token::{Token, TokenKind};
use crate::err::{Result, Error};

fn identifier_or_keyword(s: String) -> TokenKind {
    match s.as_str() {
        "var" => TokenKind::Var,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "return" => TokenKind::Return,
        "func" => TokenKind::Func,
        "class" => TokenKind::Class,
        "this" => TokenKind::This,
        "null" => TokenKind::Null,
        _ => TokenKind::Identifier(s)
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

    pub fn lex(mut self) -> Result<Vec<Token>> {
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

    fn parse_string_literal(&mut self) -> Result<TokenKind> {
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
                '"' => {
                    if s.len() >= u16::MAX as usize {
                        return Err(Error::new(format!("constant string too long, length: {}", s.len()), off));
                    }
                    return Ok(TokenKind::String(s))
                },
                _ => s.push(c)
            }
        }

        Err(Error::new("unclosed string literal".to_owned(), off - 1))
    }

    fn parse_long_double(&mut self, first: char) -> std::result::Result<TokenKind, String> {
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

        while let Some(c) = self.peek() && (c.is_ascii_alphanumeric() || c == b'_') {
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


#[cfg(test)]
mod tests {
    use crate::lexer::{Error, Lexer};
    use crate::token::{Token, TokenKind};

    fn parse(s: &str) -> Result<Vec<Token>, Error> {
        Lexer::new(s.as_bytes()).lex()
    }

    #[test]
    fn test_ok() {
        let toks = parse(r#"var a = 1.2;
if (true) {}
        "#);

        assert!(toks.is_ok());

        assert_eq!(toks.unwrap(), vec![
            Token::new(TokenKind::Var, 0),
            Token::new(TokenKind::Identifier("a".to_owned()), 4),
            Token::new(TokenKind::Eq, 6),
            Token::new(TokenKind::Double(1.2), 8),
            Token::new(TokenKind::Semi, 11),
            Token::new(TokenKind::If, 13),
            Token::new(TokenKind::LParen, 16),
            Token::new(TokenKind::True, 17),
            Token::new(TokenKind::RParen, 21),
            Token::new(TokenKind::LBrace, 23),
            Token::new(TokenKind::RBrace, 24),
        ]);

        let toks = parse(r#""abc\ndef\rg\\h" 123 // def
f();
        "#);

        assert!(toks.is_ok());
        assert_eq!(toks.unwrap(), vec![
            Token::new(TokenKind::String("abc\ndef\rg\\h".to_owned()),0),
            Token::new(TokenKind::Long(123), 17),
            Token::new(TokenKind::Identifier("f".to_owned()), 28),
            Token::new(TokenKind::LParen, 29),
            Token::new(TokenKind::RParen, 30),
            Token::new(TokenKind::Semi, 31),
        ]);
    }

    #[test]
    fn test_err() {
        let toks = parse(r#""abcdef"#);
        assert!(toks.is_err());
        assert_eq!(toks.err().unwrap(), Error::new("unclosed string literal".to_owned(), 0));

        let toks = parse("var a &= 1;");
        assert!(toks.is_err());
        assert_eq!(toks.err().unwrap(), Error::new("not support single &".to_owned(), 6));

        let toks = parse(r#""abcdef\"#);
        assert!(toks.is_err());
        assert_eq!(toks.err().unwrap(), Error::new("escape sequence: no char found after: \\".to_owned(), 7));

        let toks = parse(r#""abcdef\d"#);
        assert!(toks.is_err());
        assert_eq!(toks.err().unwrap(), Error::new("unsupport escape sequence: \\d".to_owned(), 7));
    }
}