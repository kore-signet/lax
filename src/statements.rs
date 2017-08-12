use super::{Token,Expr};
use std::rc::Rc;
use std::fmt;

#[derive(Debug,Clone)]
pub enum Statement {
    Expression(Expr),
    Variable(Token,Option<Expr>),
    Block(Vec<Rc<Statement>>),
    If(Expr,Rc<Statement>,Option<Rc<Statement>>),
    While(Expr,Rc<Statement>),
    Function(Token,Vec<Token>,Rc<Statement>),
    Return(Token,Expr),
    Import(Token)
}

impl fmt::Display for Statement {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Statement::Expression(ref e) => { write!(f,"{};",e) },
            &Statement::Variable(ref t,ref e) => { write!(f,"var {} = {:?};",&t.lexeme,e) },
            &Statement::Block(ref l) => {
                writeln!(f,"{{")?;
                for s in l {
                    writeln!(f,"{}",&s)?;
                }
                writeln!(f,"}}")
            },
            &Statement::If(ref c,ref t,ref e) => {
                writeln!(f,"if {}",c)?;
                writeln!(f,"{}",t)?;
                if let &Some(ref e_branch) = e {
                    writeln!(f,"else")?;
                    writeln!(f,"{}",e_branch)?;
                }
                Ok(())
            },
            &Statement::While(ref c,ref b) => {
                writeln!(f,"while {}",c)?;
                writeln!(f,"{}",b)
            },
            &Statement::Function(ref t,_,_) => writeln!(f,"<fn {}>",t.lexeme),
            &Statement::Return(_,ref e) => writeln!(f,"return {}",e),
            &Statement::Import(ref t) => writeln!(f,"import {}",t.lexeme)
        }
    }
}
