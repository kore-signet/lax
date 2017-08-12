use super::*;
use std::rc::Rc;

pub enum FunKind {
    Function,
    Method
}

macro_rules! try_sync {
    ($self:expr,$e:expr) => {
        match $e {
            Ok(k) => k,
            Err(e) => {
                $self.sync();
                return Err(e);
            }
        }
    }
}

pub struct Parser {
    current: usize,
    tokens: Vec<Token>
}

type ParseResult = Result<Expr,LoxError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser{
        Parser { current: 0, tokens: tokens }
    }

    pub fn parse(&mut self) -> Result<Vec<Rc<Statement>>,LoxError> {
//        println!("{:?}",&self.tokens);
        let mut statements: Vec<Rc<Statement>> = Vec::new();

        while !self.is_end() {
            statements.push(Rc::new(self.declaration()?));
        }
        Ok(statements)
    }

    fn sync(&mut self) {
        self.advance();
        while !self.is_end() {
            if self.previous().token == TokenType::Semicolon { return (); }
            match self.peek().token {
                TokenType::Class => return (),
                TokenType::Fun => return (),
                TokenType::Var => return (),
                TokenType::For => return (),
                TokenType::If => return (),
                TokenType::While => return (),
                TokenType::Import => return (),
                TokenType::Return => return (),
                _ => ()
            };
            self.advance();
        }
    }

    // Helper methods
    fn match_t(&mut self,types: Vec<TokenType>) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self,t: TokenType,msg: String) -> Result<Token,LoxError> {
        if self.check(t) {
            Ok(self.advance())
        } else {
            Err(LoxError::new(msg,self.peek().line))
        }
    }

    fn check(&mut self,t: TokenType) -> bool {
        if self.is_end() { return false; }
        self.peek().token == t
    }

    fn advance(&mut self) -> Token {
        if !self.is_end() { self.current += 1; }
        self.previous()
    }

    fn peek(&mut self) -> Token {
        self.tokens[self.current].clone()
        // This probably shouldn't clone, but im tired AND lazy.
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_end(&mut self) -> bool {
        self.peek().token == TokenType::EOF
    }

    // Grammar rules

    fn declaration(&mut self) -> Result<Statement,LoxError> {
        if self.match_t(vec![TokenType::Var]) {
            Ok(try_sync!(self,self.var_statement()))
        } else {
            Ok(try_sync!(self,self.statement()))
        }
    }

    fn var_statement(&mut self) -> Result<Statement,LoxError> {
        let name = self.consume(TokenType::Identifier,"Expected variable name".to_string())?;
        let mut initializer: Option<Expr> = None;
        if self.match_t(vec![TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon,"Expected ';' after variable declaration".to_string())?;
        Ok(Statement::Variable(name,initializer))
    }

    fn statement(&mut self) -> Result<Statement,LoxError> {
        if self.match_t(vec![TokenType::LeftBrace]) {
            self.block_statement()
        } else if self.match_t(vec![TokenType::If]) {
            self.if_statement()
        } else if self.match_t(vec![TokenType::While]) {
            self.while_statement()
        } else if self.match_t(vec![TokenType::For]) {
            self.for_statement()
        } else if self.match_t(vec![TokenType::Fun]) {
            self.function(FunKind::Function)
        } else if self.match_t(vec![TokenType::Return]) {
            self.return_statement()
        } else if self.match_t(vec![TokenType::Import]) {
            self.import_statement()
        } else {
            self.expr_statement()
        }
    }

    fn import_statement(&mut self) -> Result<Statement,LoxError> {
        let file = self.consume(TokenType::String,"Expected 'string' after 'import'".to_string())?;
        self.consume(TokenType::Semicolon,"Expected ';' after import statement".to_string())?;
        Ok(Statement::Import(file))
    }

    fn function(&mut self, kind: FunKind) -> Result<Statement,LoxError> {
        let name = self.consume(TokenType::Identifier, "Expected function/method name.".to_string())?;
        self.consume(TokenType::LeftParenthesis,"Expected '(' after fun name declaration".to_string())?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParenthesis) {
            parameters.push(self.consume(TokenType::Identifier,"Expected parameter name".to_string())?);
            while self.match_t(vec![TokenType::Comma]) {
                parameters.push(self.consume(TokenType::Identifier,"Expected parameter name".to_string())?);
            }
        }

        self.consume(TokenType::RightParenthesis,"Expected ')' after parameters.".to_string())?;
        self.consume(TokenType::LeftBrace,"Expected '{' before function/method body".to_string())?;
        let body = self.block_statement()?;
        Ok(Statement::Function(name,parameters,Rc::new(body)))
    }

    fn return_statement(&mut self) -> Result<Statement,LoxError> {
        let t = self.previous();
        let v = if !self.check(TokenType::Semicolon) { self.expression()? } else { Expr::Literal(LoxType::Nil) };
        self.consume(TokenType::Semicolon,"Expected ';' after return value".to_string())?;
        Ok(Statement::Return(t,v))
    }

    fn for_statement(&mut self) -> Result<Statement,LoxError> {
        self.consume(TokenType::LeftParenthesis, "Expected '(' after 'for'".to_string())?;

        let initializer = if self.match_t(vec![TokenType::Semicolon]) {
            None
        } else if self.match_t(vec![TokenType::Var]) {
            Some(self.var_statement()?)
        } else {
            Some(self.expr_statement()?)
        };

        let cond = if !self.check(TokenType::Semicolon) { self.expression()? } else { Expr::Literal(LoxType::Boolean(true)) };

        self.consume(TokenType::Semicolon, "Expected ';' after loop condition".to_string())?;

        let increment = if !self.check(TokenType::RightParenthesis) { Some(self.expression()?) } else { None };

        self.consume(TokenType::RightParenthesis, "Expected ')' after for clauses".to_string())?;

        let mut body = self.statement()?;

        if let Some(i) = increment {
            body = Statement::Block(vec![Rc::new(body),Rc::new(Statement::Expression(i))]);
        }

        body = Statement::While(cond,Rc::new(body));

        if let Some(init) = initializer {
            body = Statement::Block(vec![Rc::new(init),Rc::new(body)]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Statement,LoxError> {
        self.consume(TokenType::LeftParenthesis,"Expected '(' after while".to_string())?;
        let e = self.expression()?;
        self.consume(TokenType::RightParenthesis,"Expected ')' after while".to_string())?;
        let body = self.statement()?;

        Ok(Statement::While(e,Rc::new(body)))
    }

    fn if_statement(&mut self) -> Result<Statement,LoxError> {
        self.consume(TokenType::LeftParenthesis,"Expected '(' after 'if'".to_string())?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParenthesis,"Expected ')' after 'if' condition".to_string())?;

        let then = Rc::new(self.statement()?);
        let or = if self.match_t(vec![TokenType::Else]) {
            Some(Rc::new(self.statement()?))
        } else {
            None
        };
        Ok(Statement::If(cond,then,or))
    }

    fn block_statement(&mut self) -> Result<Statement,LoxError> {
        let mut statements: Vec<Rc<Statement>> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            statements.push(Rc::new(self.declaration()?));
        }

        self.consume(TokenType::RightBrace,"Expected '}' after block".to_string())?;
        Ok(Statement::Block(statements))
    }

    fn expr_statement(&mut self) -> Result<Statement,LoxError> {
        let e = self.expression()?;
        self.consume(TokenType::Semicolon,"Expected a ';' after expression".to_string())?;
        Ok(Statement::Expression(e))
    }

    fn expression(&mut self) -> ParseResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult {
        let e = self.or()?;

        if self.match_t(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = e {
                return Ok(Expr::Assign(name,Rc::new(value)));
            }

            return Err(LoxError::new("Invalid assignment target".to_string(),equals.line));
        }

        Ok(e)
    }

    fn or(&mut self) -> ParseResult {
        let mut e = self.and()?;
        while self.match_t(vec![TokenType::Or]) {
            let op = self.previous();
            let right = self.and()?;
            e = Expr::Logical(Rc::new(e),op,Rc::new(right));
        }
        Ok(e)
    }

    fn and(&mut self) -> ParseResult {
        let mut e = self.equality()?;
        while self.match_t(vec![TokenType::And]) {
            let op = self.previous();
            let right = self.and()?;
            e = Expr::Logical(Rc::new(e),op,Rc::new(right));
        }
        Ok(e)
    }

    fn equality(&mut self) -> ParseResult {
        let mut e = self.comparison()?;
        while self.match_t(vec![TokenType::BangEqual,TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            e = Expr::Binary(Rc::new(e),op,Rc::new(right));
        }

        Ok(e)
    }

    fn comparison(&mut self) -> ParseResult {
        let mut e = self.addition()?;

        while self.match_t(vec![TokenType::Greater,TokenType::GreaterEqual,TokenType::Less,TokenType::LessEqual]) {
            let op = self.previous();
            let right = self.addition()?;
            e = Expr::Binary(Rc::new(e),op,Rc::new(right));
        }

        Ok(e)
    }

    fn addition(&mut self) -> ParseResult {
        let mut e = self.multiplication()?;

        while self.match_t(vec![TokenType::Minus,TokenType::Plus]) {
            let op = self.previous();
            let right = self.multiplication()?;
            e = Expr::Binary(Rc::new(e),op,Rc::new(right));
        }

        Ok(e)
    }

    fn multiplication(&mut self) -> ParseResult {
        let mut e = self.unary()?;

        while self.match_t(vec![TokenType::Slash,TokenType::Star]) {
            let op = self.previous();
            let right = self.unary()?;
            e = Expr::Binary(Rc::new(e),op,Rc::new(right));
        }

        Ok(e)
    }

    fn unary(&mut self) -> ParseResult {
        if self.match_t(vec![TokenType::Bang,TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(op,Rc::new(right)));
        }

        self.call()
    }

    fn call(&mut self) -> ParseResult {
        let mut e = self.primary()?;
        if self.match_t(vec![TokenType::LeftParenthesis]) {
            e = self.finish_call(e)?;
        }
        Ok(e)
    }

    fn finish_call(&mut self,expr: Expr) -> ParseResult {
        let mut arguments: Vec<Rc<Expr>> = Vec::new();
        if !self.check(TokenType::RightParenthesis) {
            arguments.push(Rc::new(self.expression()?));
            while self.match_t(vec![TokenType::Comma]) {
                arguments.push(Rc::new(self.expression()?));
            }
        }

        let paren = self.consume(TokenType::RightParenthesis,"Expected ')' after arguments".to_string())?;
        Ok(Expr::Call(Rc::new(expr),paren,arguments))
    }

    // End of grammar (finally)
    fn primary(&mut self) -> ParseResult {
        if self.match_t(vec![TokenType::False]) { return Ok(Expr::Literal(LoxType::Boolean(false))) }
        if self.match_t(vec![TokenType::True]) { return Ok(Expr::Literal(LoxType::Boolean(true))) }
        if self.match_t(vec![TokenType::Nil]) { return Ok(Expr::Literal(LoxType::Nil)) }

        if self.match_t(vec![TokenType::Number,TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.unwrap()))
        }

        if self.match_t(vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()))
        }

        if self.match_t(vec![TokenType::LeftParenthesis]) {
            let e = self.expression()?;
            self.consume(TokenType::RightParenthesis,"Expected ')' after expression".to_string())?;
            return Ok(Expr::Grouping(Rc::new(e)))
        };
        Err(LoxError::new("Expected expression".to_string(),self.peek().line))
    }
}
