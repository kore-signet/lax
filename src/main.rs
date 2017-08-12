extern crate lax;
extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use lax::{LoxType,Clock};
use lax::lox_std;
use lax::scanner::Scanner;
use lax::parser::Parser;
use lax::interpreter::Interpreter;
use std::io::{self,Read};
use std::boxed::Box;
use std::fs::File;


use docopt::Docopt;

const USAGE: &'static str = "
lax, the simple Rust Lox interpreter

Usage:
    lax run <file>
    lax -c <code>
    lax (-h | --help)
    lax

Options:
    -h --help   Show this screen
    -c  Interpret string passed directly
";

#[derive(Deserialize)]
struct Args {
    cmd_run: bool,
    arg_file: String,
    arg_code: String,
    flag_c: bool
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    let mut interpreter = Interpreter::new();

    interpreter.env.define("clock",LoxType::Callable(Box::new(Clock)));
    interpreter.env.define("print",LoxType::Callable(Box::new(lox_std::Print)));
    interpreter.env.define("readline",LoxType::Callable(Box::new(lox_std::Readline)));

    if args.flag_c {
        run(args.arg_code,&mut interpreter);
    } else if args.cmd_run {
        let mut buffer = String::new();
        File::open(args.arg_file).unwrap().read_to_string(&mut buffer).unwrap();
        run(buffer,&mut interpreter);
    } else {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            run(buffer,&mut interpreter);
        }
    }
}

fn run(s: String,i: &mut Interpreter) {
    let mut scanner = Scanner::new(s);
    scanner.scan().unwrap();
    let mut parser = Parser::new(scanner.tokens);
    let ast = parser.parse().unwrap();
    i.interpret(&ast).unwrap();
}
