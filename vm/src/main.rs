use std::{env, fs};
use std::error::Error;
use std::process::exit;
use common::constant::MAGIC;
use common::loader::Loader;
use compilerlib::code_gen::check_and_gen;
use compilerlib::lexer::Lexer;
use compilerlib::parser::Parser;
use crate::runtime::exec;

mod value;
mod stack;
mod runtime;
mod ffi;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: charon [charon byte code file path]");
        exit(1);
    }

    if let Err(e) = run(unsafe {args.get_unchecked(1)}) {
        eprintln!("{e}");
        exit(1);
    }
}

fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let mut bytes = fs::read(path)?;
    
    if !is_bytecode(&bytes) {
        let tokens = Lexer::new(&bytes).lex()?;
        drop(bytes);
        let program = Parser::new(tokens).parse()?;
        bytes = check_and_gen(&program)?;
    }
    
    let program = Loader::new(&bytes).load()?;
    drop(bytes);
    
    exec(program).map_err(|e| e.into())
}

fn is_bytecode(bytes: &[u8]) -> bool {
    let len = MAGIC.len();
    if bytes.len() < len {
        false
    } else {
        &bytes[..len] == MAGIC.as_bytes()
    }
}