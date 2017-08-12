use std::io;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct LoxError {
    line: i32,
    err: String,
    lower: Option<io::Error>
}

impl Error for LoxError {
    fn description(&self) -> &str {
        &self.err
    }

    fn cause(&self) -> Option<&Error> {
        if let Some(ref err) = self.lower {
            Some(err as &Error)
        } else {
            None
        }
    }
}

impl fmt::Display for LoxError {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,"Error: {} in line {}",&self.err,&self.line)
    }
}

impl LoxError {
    pub fn new(s: String,l: i32) -> LoxError {
        LoxError {
            line: l,
            err: s,
            lower: None
        }
    }

    pub fn with_lower(s: String,l: i32,e: io::Error) -> LoxError {
        LoxError {
            line: l,
            err: s,
            lower: Some(e)
        }
    }
}
