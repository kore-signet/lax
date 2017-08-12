use super::interpreter::Interpreter;
use super::{LoxError,LoxType};
use std::fmt::Debug;
use std::cmp::Ordering;

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(&self, interpreter: &mut Interpreter,arguments: Vec<LoxType>) -> (Interpreter,Result<LoxType,LoxError>);

    fn box_clone(&self) -> Box<Callable>;
}

impl Clone for Box<Callable> {
    fn clone(&self) -> Box<Callable> {
        self.box_clone()
    }
}

impl PartialEq for Box<Callable> {
    fn eq(&self,other: &Box<Callable>) -> bool {
        false
    }
}

impl PartialOrd for Box<Callable> {
    fn partial_cmp(&self,other: &Box<Callable>) -> Option<Ordering> {
        None
    }
}
