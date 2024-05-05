use std::{env, fs};
use std::env::current_dir;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use common::err_println;
use compilerlib::code_gen::check_and_gen;
use compilerlib::{lex, parse};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        err_println("usage: charonc [charon source file path]");
        exit(1);
    }

    let sourcecode_path = unsafe {args.get_unchecked(1)};
    
    if let Err(e) = run(sourcecode_path) {
        err_println(&format!("{e}"));
        exit(1);
    }
}

fn run(sourcecode_path: &str) -> Result<(), Box<dyn Error>> {
    let bytes = match fs::read(sourcecode_path) {
        Ok(v) => v,
        Err(e) => {
            err_println(&format!("failed to read source code file: {sourcecode_path}, with error: {e}"));
            exit(1);
        }
    };
    
    let output_path = output_path(sourcecode_path)?;
    
    let tokens = lex(&bytes)?;
    let program = parse(tokens, &bytes)?;
    drop(bytes);
    let bytecode = check_and_gen(&program)?;
    fs::write(output_path, bytecode)?;
    Ok(())
}

fn output_path(sourcecode_path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(sourcecode_path);
    let Some(name) = path.file_stem() else {
        return Err(format!("failed to get {sourcecode_path}'s name").into());
    };
    let mut name = name.to_os_string();
    name.push(".charonbc");
    let curr_dir = current_dir()?;
    Ok(curr_dir.join(name))
}