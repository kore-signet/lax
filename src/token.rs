use std::fmt;
use super::{LoxType,TokenType};

#[derive(Debug,Clone)]
pub struct Token {
    pub token: TokenType,
    pub lexeme: String,
    pub line: i32,
    pub literal: Option<LoxType>
}

