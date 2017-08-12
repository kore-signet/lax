#![feature(ascii_ctype)]
#![feature(try_from)]

#[macro_use]
extern crate lazy_static;
extern crate chrono;

mod token_type;
mod lox_type;
mod err;
mod token;
mod expr;
mod statements;
mod environment;
mod callable;
mod clock;
mod fun;
pub use fun::*; 
pub use clock::*;
pub use callable::*;
pub use environment::*;
pub use statements::*;
pub use lox_type::*;
pub use expr::*;
pub use token_type::*;
pub use err::*;
pub use token::*;
pub mod lox_std;
pub mod scanner;
pub mod parser;
pub mod interpreter;
// TRIPLE THREAT
