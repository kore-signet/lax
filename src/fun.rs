use super::{Callable,LoxError,Statement,LoxType,Environment};
use interpreter::{InterpreterError,Interpreter};
use std::rc::Rc;
use std::boxed::Box;
use std::fmt;
use std::borrow::Borrow;

#[derive(Clone)]
pub struct LoxFun {
    declaration: Rc<Statement>
}

impl LoxFun {
    pub fn new(declaration: Rc<Statement>) -> LoxFun {
        LoxFun {
            declaration: declaration
        }
    }
}

impl Callable for LoxFun {
    fn arity(&self) -> usize {
        match self.declaration.borrow() {
            &Statement::Function(_,ref args,_) => args.len(),
            _ => 0
        }
    }

    fn call(&self,i: &mut Interpreter,arguments: Vec<LoxType>) -> (Interpreter, Result<LoxType,LoxError>) {
        let mut env = Environment::with_enclosing(i.env.clone());

        let (args,statements) = match self.declaration.borrow() {
            &Statement::Function(_,ref args,ref statements) => {
                (args,statements)
            },
            _ => panic!("Tryed to call an invalid function")
        };

        for (n,arg) in arguments.into_iter().enumerate() {
            env.define(args[n].lexeme.as_str(),arg);
        }

        let statements = if let &Statement::Function(_,_,ref block_statement) = self.declaration.borrow() {
            if let &Statement::Block(ref statements_) = block_statement.borrow() {
                statements_.clone()
            } else {
                panic!("Tried to call an invalid function");
            }
        } else {
            panic!("Tried to call an invalid function")
        };

        let res = i.interpret_block(&statements,Some(env));
        //println!("got res{:?}",&res);
        match res {
            Ok(_) => (i.clone(),Ok(LoxType::Nil)),
            Err(e) => {
            //    println!("err {:?}",e);
                match e {
                    InterpreterError::LoxError(e_) => (i.clone(),Err(e_)),
                    InterpreterError::Return(v) => (i.clone(),Ok(v)),
                    _ => panic!("what") // user friendly messages, everyone
                }
            }
        }
    }

    fn box_clone(&self) -> Box<Callable> {
        Box::new((*self).clone())
    }
}

impl fmt::Debug for LoxFun {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self.declaration.borrow() {
            &Statement::Function(ref t,_,_) => write!(f,"<function {}>",t.lexeme),
            _ => write!(f,"Invalid function")
        }
    }
}
