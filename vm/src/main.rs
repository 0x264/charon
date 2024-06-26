use std::{env, fs};
use std::error::Error;
use std::process::exit;
use common::constant::MAGIC;
use common::err_println;
use common::loader::Loader;
use compilerlib::code_gen::check_and_gen;
use compilerlib::{lex, parse};
use crate::runtime::exec;

mod value;
mod stack;
mod runtime;
mod ffi;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        err_println("usage: charon [charon byte code file path]");
        exit(1);
    }

    if let Err(e) = run(unsafe {args.get_unchecked(1)}) {
        err_println(&format!("{e}"));
        exit(1);
    }
}

fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let mut bytes = fs::read(path)?;
    
    if !is_bytecode(&bytes) {
        let tokens = lex(&bytes)?;
        let program = parse(tokens, &bytes)?;
        drop(bytes);
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