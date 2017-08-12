use super::{TokenType,LoxError,Token,LoxType};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::io;

lazy_static! {
            static ref KEYWORDS: HashMap<&'static str,TokenType> = {
                let mut m: HashMap<&'static str,TokenType> = HashMap::new();
                m.insert("and",TokenType::And);
                m.insert("class",TokenType::Class);
                m.insert("else",TokenType::Else);
                m.insert("false",TokenType::False);
                m.insert("for",TokenType::For);
                m.insert("fun",TokenType::Fun);
                m.insert("if",TokenType::If);
                m.insert("nil",TokenType::Nil);
                m.insert("or",TokenType::Or);
                m.insert("return",TokenType::Return);
                m.insert("super",TokenType::Super);
                m.insert("this",TokenType::This);
                m.insert("true",TokenType::True);
                m.insert("var",TokenType::Var);
                m.insert("while",TokenType::While);
                m.insert("import",TokenType::Import);
                m
            };
}

pub struct Scanner {
    source: Vec<char>,
    pub tokens: Vec<Token>,
    current: i32,
    line: i32,
    start: i32
}

macro_rules! add_match {
    ($self:expr,$expected:expr,$first:expr,$last:expr) => {
        if $self.match_c($expected) {
            $self.add($first);
        } else {
            $self.add($last);
        }
    }
}

impl Scanner {
    pub fn new(s: String) -> Scanner {
        Scanner {
            source: s.chars().collect::<Vec<char>>(),
            tokens: Vec::new(),
            current: 0,
            line: 0,
            start: 0
        }
    }

    pub fn scan(&mut self) -> Result<(),Vec<LoxError>> {
        let mut errors: Vec<LoxError> = Vec::new();
        let mut failed = false;
        while !(self.is_end()) {
            self.start = self.current;
            if let Err(errs)  = self.scan_token() {
                errors.extend(errs);
                failed = true;
            }
        }
        self.tokens.push(Token { token: TokenType::EOF, lexeme:"".to_string(), line:self.line, literal: None });

        if failed {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn scan_token(&mut self) -> Result<(),Vec<LoxError>> {
        let mut errs: Vec<LoxError> = Vec::new();
        let mut failed = false;
        let c = self.advance();
//        println!("{}",&c);
        match c {
            '(' => { self.add(TokenType::LeftParenthesis); },
            ')' => { self.add(TokenType::RightParenthesis); },
            '{' => { self.add(TokenType::LeftBrace); },
            '}' => { self.add(TokenType::RightBrace); },
            ',' => { self.add(TokenType::Comma); },
            '.' => { self.add(TokenType::Dot); },
            '-' => { self.add(TokenType::Minus); },
            '+' => { self.add(TokenType::Plus); },
            ';' => { self.add(TokenType::Semicolon); },
            '*' => { self.add(TokenType::Star); },
            '!' => { add_match!(self,'=',TokenType::BangEqual,TokenType::Bang); },
            '=' => { add_match!(self,'=',TokenType::EqualEqual,TokenType::Equal); },
            '<' => { add_match!(self,'=',TokenType::LessEqual,TokenType::Less); },
            '>' => { add_match!(self,'=',TokenType::GreaterEqual,TokenType::Greater); },
            '/' => {
                if self.match_c('/') {
                    while self.peek() != '\n' && !(self.is_end()) {
                        self.advance();
                    }
                } else {
                    self.add(TokenType::Slash);
                }
            },
            '"' => {
                if let Err(e) = self.string() {
                    errs.push(e);
                }
            },
            '\n' => { self.line += 1; },
            ' ' => (),
            '\r' => (),
            '\t' => (),
            _ => {
                if c.is_ascii_digit() {
                    if let Err(e) = self.number() {
                        errs.push(e);
                        failed = true;
                    };
                } else if c.is_alphabetic() || c == '_' || c == '-' {
                    self.identifier();
                } else {
                    errs.push(LoxError::new("Unexpected character".to_string(),self.line));
                }
            }
        };

        if failed {
            Err(errs)
        } else {
            Ok(())
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[(self.current - 1) as usize]
    }

    fn add(&mut self,t: TokenType) {
  //      println!("{:?}",t);
        let lexeme = (&self.source[self.start as usize..self.current as usize]).iter().collect::<String>();
        self.tokens.push(Token { token: t, lexeme: lexeme, line: self.line, literal: None });
    }

    fn add_token(&mut self,t: TokenType,l: LoxType) {
        let lexeme = (&self.source[self.start as usize..self.current as usize]).iter().collect::<String>();
        self.tokens.push(Token { token: t, lexeme: lexeme, line: self.line, literal: Some(l) });
    }

    fn match_c(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }
        if self.source[self.current as usize] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&mut self) -> char {
        if self.is_end() {
            '\0'
        } else {
            self.source[self.current as usize]
        }
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= (self.source.len() as i32) {
            '\0'
        } else {
            self.source[(self.current + 1) as usize]
        }
    }

    fn string(&mut self) -> Result<(),LoxError> {
        while self.peek() != '"' && !(self.is_end()) {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            return Err(LoxError::new("Unterminated string".to_string(),self.line));
        }

        self.advance();
        let s = (&self.source[(self.start + 1) as usize..(self.current - 1) as usize]).iter().collect::<String>();
        self.add_token(TokenType::String,LoxType::String(s));
        Ok(())
    }

    fn number(&mut self) -> Result<(),LoxError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let s = &self.source[self.start as usize..self.current as usize].iter().collect::<String>();
        match s.parse::<f64>() {
            Ok(n) => self.add_token(TokenType::Number,LoxType::Number(n)),
            Err(e) => return Err(LoxError::with_lower("Invalid number".to_string(),self.line,io::Error::new(io::ErrorKind::Other,e)))
        };
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '-' {
            self.advance();
        }
        let s = (&self.source[self.start as usize..self.current as usize]).iter().collect::<String>();
        match KEYWORDS.get(s.as_str()) {
            Some(t) => { self.add(t.clone()) },
            None => self.add(TokenType::Identifier)
        };
    }

    fn is_end(&self) -> bool {
        self.current >= (self.source.len() as i32)
    }
}
