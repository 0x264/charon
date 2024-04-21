#![feature(let_chains)]


use std::error::Error;
use crate::lexer::Lexer;
use crate::parser::Parser;

mod token;
mod lexer;
mod parser;
mod ast;
mod err;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let code = r#"
        class abc {
            func haha(x, y) {
                var a = 1;
                var b;
                while (true) {
                    a + 1;
                    if (a == 10) {
                        b = 1;
                        return b;
                    } else if (a == 8) {
                        b = 2;
                    } else {
                        b = 3;
                    }
                }
            }
        }

        var a = 2;
        func f() {}
        func ff(a, b) {return a + b;}

        ff(12, a);
    "#;

    let tokens = Lexer::new(code.as_bytes()).lex()?;
    let program = Parser::new(tokens).parse()?;
    println!("{program:?}");
    Ok(())
}