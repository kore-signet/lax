use super::{Token,LoxType};
use std::rc::Rc;
use std::fmt;

#[derive(Clone,Debug)]
pub enum Expr {
    Binary(Rc<Expr>,Token,Rc<Expr>),
    Grouping(Rc<Expr>),
    Literal(LoxType),
    Unary(Token,Rc<Expr>),
    Variable(Token),
    Assign(Token,Rc<Expr>),
    Logical(Rc<Expr>,Token,Rc<Expr>),
    Call(Rc<Expr>,Token,Vec<Rc<Expr>>)
}

impl fmt::Display for Expr {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expr::Binary(ref left,ref token,ref right) => {
                write!(f,"({} {} {})",&token.lexeme,left,right)
            },
            &Expr::Grouping(ref e) => {
                write!(f,"(group {})",e)
            },
            &Expr::Literal(ref l) => {
                write!(f,"{}",l)
            },
            &Expr::Unary(ref token,ref e) => {
                write!(f,"({} {})",&token.lexeme,e)
            },
            &Expr::Variable(ref token) => {
                write!(f,"var({})",&token.lexeme)
            },
            &Expr::Assign(ref token,ref e) => {
                write!(f,"{} = {}",&token.lexeme,e)
            },
            &Expr::Logical(ref left,ref token,ref right) => {
                write!(f,"({} {} {})",left,&token.lexeme,right)
            },
            &Expr::Call(ref calle,_,ref args) => {
                write!(f,"{}(",calle)?;
                for a in args {
                    write!(f,"{},",a)?;
                }
                write!(f,")")
            }
        }
    }
}

