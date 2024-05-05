#![feature(let_chains)]

use std::error::Error;
use common::line_column_info::LineColumnInfo;
use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;

mod token;
mod lexer;
mod parser;
mod ast;
mod err;
pub mod code_gen;

pub fn lex(bytes: &[u8]) -> Result<Vec<Token>, Box<dyn Error>> {
    Lexer::new(bytes).lex().map_err(|e| { map_err(e, bytes).into() })
}

pub fn parse(tokens: Vec<Token>, bytes: &[u8]) -> Result<Program, Box<dyn Error>> {
    Parser::new(tokens).parse().map_err(|e| { map_err(e, bytes).into() })
}

fn map_err(e: err::Error, bytes: &[u8]) -> String {
    let line_column_info = LineColumnInfo::new(&bytes);
    let (line, column) = line_column_info.line_column_info(e.offset);
    format!("{}, ({line}: {column})", e.msg)
}