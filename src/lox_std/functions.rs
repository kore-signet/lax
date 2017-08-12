use super::super::{LoxType,LoxError,Callable};
use interpreter::Interpreter;
use std::io::{self,Read};
use std::boxed::Box;

macro_rules! callable_fn {
    ($fn:expr,$name:tt,$arity:expr) => {
            #[derive(Debug,Clone)]
            pub struct $name;

            impl Callable for $name {
                fn arity(&self) -> usize {
                    $arity
                }

                fn call(&self,i: &mut Interpreter,arguments: Vec<LoxType>) -> (Interpreter,Result<LoxType,LoxError>) {
                    $fn(i,arguments)
                }

                fn box_clone(&self) -> Box<Callable> {
                    Box::new((*self).clone())
                }
            }
    }
}

callable_fn!(|i: &mut Interpreter,args: Vec<LoxType>| {
    println!("{}",args[0]);
    (i.clone(),Ok(LoxType::Nil))
},Print,1);

callable_fn!(|i: &mut Interpreter,args| {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    (i.clone(),Ok(LoxType::String(s)))
},Readline,0);
