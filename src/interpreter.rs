use super::*;
use scanner::*;
use parser::*;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::convert::{TryInto,TryFrom};
use std::borrow::Borrow;
use std::boxed::Box;
use std::mem;
// This is... weird
#[derive(Debug)]
pub enum InterpreterError {
    LoxError(LoxError),
    LoxErrors(Vec<LoxError>),
    Return(LoxType)
}

impl From<LoxError> for InterpreterError {
    fn from(v: LoxError) -> InterpreterError {
        InterpreterError::LoxError(v)
    }
}

impl From<Vec<LoxError>> for InterpreterError {
    fn from(v: Vec<LoxError>) -> InterpreterError {
        InterpreterError::LoxErrors(v)
    }
}

#[derive(Clone)]
pub struct Interpreter { pub env: Box<Environment> }

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            env: Box::new(Environment::new())
        }
    }

    pub fn import(&mut self,file: Token) -> Result<(),InterpreterError> {
        let mut s = String::new();
        match File::open(String::try_from(file.literal.unwrap()).unwrap()) {
            Ok(mut k) => k.read_to_string(&mut s),
            Err(e) => return Err(InterpreterError::LoxError(LoxError::with_lower("Imported file couldn't be opened".to_string(),file.line,e)))
        };

        let env = {
            let mut scanner = Scanner::new(s);
            scanner.scan()?;
            let mut parser = Parser::new(scanner.tokens);
            let ast = parser.parse()?;
            let mut i = Interpreter::new();
            i.interpret(&ast);
            i.env
        };

        self.env.extend(env);
        Ok(())
    }

    pub fn interpret_block(&mut self,statements: &Vec<Rc<Statement>>, environment: Option<Environment>) -> Result<(),InterpreterError> {
    	let new = match environment {
            Some(e) => Box::new(Environment::with_enclosing(Box::new(e))),
            None => Box::new(Environment::with_enclosing(self.env.clone()))
        };
        self.env = new;
        let res = self.interpret(statements);
    	self.env = self.env.clone().enclosing.unwrap();
        res
    }

    pub fn interpret(&mut self,statements: &Vec<Rc<Statement>>) -> Result<(),InterpreterError> {
        'interpreter: for s in statements {
            match s.borrow() {
                &Statement::Expression(ref e) => { self.evaluate(&e)?; },
                &Statement::Variable(ref name,ref init) => {
                    let value = match init {
                        &Some(ref i) => self.evaluate(i)?,
                        &None => LoxType::Nil
                    };
                    self.env.define(&name.lexeme,value);
                },
                &Statement::Block(ref statements) => {
                    self.interpret_block(statements,None)?;
                },
                &Statement::If(ref cond,ref then,ref or) => {
                    if bool::from(self.evaluate(cond)?) {
                        self.interpret(&vec![then.clone()])?;
                    } else if let &Some(ref or_branch) = or {
                        self.interpret(&vec![or_branch.clone()])?;
                    }
                },
                &Statement::While(ref cond,ref body) => {
                    while bool::from(self.evaluate(cond)?) {
                        self.interpret(&vec![body.clone()])?;
                    }
                },
                &Statement::Function(ref t,_,_) => {
                    let fun = LoxFun::new(s.clone());
                    self.env.define(&t.lexeme,LoxType::Callable(Box::new(fun)))
                },
                &Statement::Return(_,ref exp) => {
                //    println!("returning");
                    return Err(InterpreterError::Return(self.evaluate(exp)?));
                },
                &Statement::Import(ref t) => {
                    self.import(t.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self,e: &Expr) -> Result<LoxType,InterpreterError> {
        match e {
            &Expr::Assign(ref t,ref v) => {
                let value = self.evaluate(v)?;
                if self.env.contains(&t.lexeme) {
                    self.env.assign(&t.lexeme,value);
                    return Ok((LoxType::Nil));
                } else {
                    return Err((InterpreterError::LoxError(LoxError::new("Undefined variable".to_string(),t.line.clone()))));
                }
            },
            &Expr::Variable(ref t) => {
                match self.env.get(&t.lexeme) {
                    Some(r) => return Ok((r)),
                    None => return Err((InterpreterError::LoxError(LoxError::new("Variable not found".to_string(),t.line.clone()))))
                }
            },
            &Expr::Logical(ref left,ref op,ref right) => {
                let left = self.evaluate(left)?;
                if op.token == TokenType::Or {
                    if bool::from(left.clone()) {
                        return Ok((left))
                    }
                } else {
                    if !bool::from(left.clone()) {
                        return Ok((left))
                    }
                }

                self.evaluate(right)
            },
            &Expr::Call(ref callee,ref paren,ref args) => {
                let fun = match self.evaluate(callee)? {
                    LoxType::Callable(c) => c,
                    _ => return Err((InterpreterError::LoxError(LoxError::new("Expected callable object but got normal type".to_string(),paren.line.clone()))))
                };

                let mut arguments: Vec<LoxType> = Vec::new();
                for a in args {
                    arguments.push(self.evaluate(a)?);
                }

                if arguments.len() != fun.arity() {
                    return Err((InterpreterError::LoxError(LoxError::new(format!("Expected {} arguments but got {}",fun.arity(),arguments.len()),paren.line.clone()))));
                }

                let (i,res) = fun.call(self,arguments);
                mem::replace(self,i);
                Ok(res?)
            }
            &Expr::Literal(ref t) => Ok(t.clone()), // the easy one
            &Expr::Grouping(ref expr) => self.evaluate(expr),
            &Expr::Unary(ref op,ref expr) => {
                let right = self.evaluate(e)?;
                match op.token {
                    TokenType::Minus => {
                        return Ok((LoxType::Number((-(f64::try_from(right)?)).clone())));
                    },
                    TokenType::Bang => {
                        return Ok((LoxType::Boolean((!(bool::try_from(right)?)).clone())));
                    },
                    _ => return Err((InterpreterError::LoxError(LoxError::new("Invalid unary operator".to_string(),0))))
                }
            },
            &Expr::Binary(ref left,ref op,ref right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match op.token.clone() {
                    TokenType::Minus => {
                        let result = f64::try_from(left)?.clone() - f64::try_from(right)?.clone();
                        return Ok((LoxType::Number(result)));
                    },
                    TokenType::Slash => {
                        let result = f64::try_from(left)?.clone() / f64::try_from(right)?.clone();
                        return Ok((LoxType::Number(result)));
                    },
                    TokenType::Star => {
                        let result = f64::try_from(left)?.clone() * f64::try_from(right)?.clone();
                        return Ok((LoxType::Number(result)));
                    },
                    TokenType::Plus => {
                        match (&left,&right) {
                            (&LoxType::Number(_),&LoxType::Number(_)) => {
                                let result = f64::try_from(left)?.clone() + f64::try_from(right)?.clone();
                                return Ok((LoxType::Number(result)));
                            },
                            (&LoxType::String(_),&LoxType::String(_)) => {
                                return Ok((LoxType::String(format!("{}{}",left,right))));
                            },
                            _ => {
                                return Err((InterpreterError::LoxError(LoxError::new("Can't add two diferent types".to_string(),0))));
                            }
                        }
                    },
                    TokenType::Greater => {
                        let result = left > right;
                        return Ok((LoxType::Boolean(result)));
                    },
                    TokenType::GreaterEqual => {
                        let result =  left  >= right;
                        return Ok((LoxType::Boolean(result)));
                    },
                    TokenType::Less => {
                        let result = left  < right;
                        return Ok((LoxType::Boolean(result)));
                    },
                    TokenType::LessEqual => {
                        let result = left  <= right;
                        return Ok((LoxType::Boolean(result)));
                    },
                    TokenType::BangEqual => {
                        return Ok((LoxType::Boolean(left != right)));
                    },
                    TokenType::EqualEqual => {
                        return Ok((LoxType::Boolean(left == right)));
                    },
                    _ => return Err((InterpreterError::LoxError(LoxError::new("Invalid binary operator".to_string(),0))))
                };
            }
        }
    }
}
