use chrono::prelude::*;
use super::{LoxError,LoxType,Callable};
use super::interpreter::Interpreter;

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize { 0 }
    fn call(&self,interpreter: &mut Interpreter,arguments: Vec<LoxType>) -> (Interpreter,Result<LoxType,LoxError>) {
        (interpreter.clone(),Ok(LoxType::Number(Local::now().timestamp() as f64)))
    }
    fn box_clone(&self) -> Box<Callable> {
        Box::new((*self).clone())
    }
}
