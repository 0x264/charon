use std::fmt::Display;
use std::io;
use std::io::IsTerminal;

pub mod opcode;
pub mod constant;
pub mod reader;
pub mod program;
pub mod loader;
pub mod line_column_info;

pub type Result<T> = std::result::Result<T, String>;

pub fn err_print<T: Display + ?Sized>(err: &T) {
    let is_terminal = io::stderr().is_terminal();
    if is_terminal {
        eprint!("[31m");
    }
    eprint!("{err}");
    if is_terminal {
        eprint!("[0m");
    }
}

pub fn err_println<T: Display + ?Sized>(err: &T) {
    err_print(err);
    eprintln!();
}