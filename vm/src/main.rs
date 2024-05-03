use std::{env, fs};
use std::error::Error;
use std::process::exit;
use common::loader::Loader;
use crate::runtime::exec;

mod value;
mod stack;
mod runtime;

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
    let bytes = fs::read(path)?;
    
    let program = Loader::new(&bytes).load()?;
    drop(bytes);
    
    exec(program).map_err(|e| e.into())
}