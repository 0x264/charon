use std::{env, fs};
use std::process::exit;
use common::err_println;
use common::loader::Loader;
use crate::disassembler::disassemble;

mod disassembler;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        err_println("usage: charonp [charon byte code file path]");
        exit(1);
    }
    
    let bytes = match fs::read(unsafe {args.get_unchecked(1)}) {
        Ok(v) => v,
        Err(e) => {
            err_println(&format!("failed to read byte code file: {e}"));
            exit(1);
        }
    };
    
    if let Err(e) = run(bytes) {
        err_println(&e);
        exit(1);
    }
}

fn run(bytes: Vec<u8>) -> Result<(), String> {
    let program = Loader::new(&bytes).load()?;
    drop(bytes);
    disassemble(&program)
}