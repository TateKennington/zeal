use std::{cell::RefCell, rc::Rc};

use crate::{
    interpreter::Environment,
    scanner::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    col: usize,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            tokens: Vec::default(),
            index: 0,
            col: 0,
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Expr> {
        self.tokens = tokens;
        let mut res = Vec::default();
        while self.peek().is_some() {
            while self.matches(vec![TokenType::LineEnd]) {}
            if self.matches(vec![TokenType::EndOfFile]) {
                break;
            }
            res.push(self.statement());
        }
        res
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if let Some(token) = self.tokens.get(self.index) {
            return token.token_type == token_type;
        }
        false
    }

    fn advance(&mut self) -> Token {
        if self.index < self.tokens.len() {
            self.index += 1;
        }
        self.previous()
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.index - 1].clone()
    }

    fn peek(&mut self) -> Option<Token> {
        if self.index >= self.tokens.len() {
            return None;
        }
        Some(self.tokens[self.index].clone())
    }

    fn peek_next(&mut self) -> Option<Token> {
        if self.index + 1 >= self.tokens.len() {
            return None;
        }
        Some(self.tokens[self.index + 1].clone())
    }

    fn matches(&mut self, tokens: Vec<TokenType>) -> bool {
        if tokens.iter().any(|t| self.check(t.clone())) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches_all(&mut self, tokens: Vec<TokenType>) -> bool {
        if self.index + tokens.len() > self.tokens.len() {
            false
        } else if tokens
            .iter()
            .enumerate()
            .all(|(i, token)| *token == self.tokens[i + self.index].token_type)
        {
            self.index += tokens.len();
            true
        } else {
            false
        }
    }

    fn matches_over_line(&mut self, token: TokenType) -> bool {
        self.matches(vec![token.clone()]) || self.matches_all(vec![TokenType::LineEnd, token])
    }

    fn block(&mut self) -> Expr {
        let mut res = Vec::default();
        while !self.matches(vec![TokenType::EndBlock]) {
            res.push(self.statement());
        }
        Expr::Block(res)
    }

    fn control_expression(&mut self) -> Expr {
        let Some(curr) = self.peek() else {
            panic!("Unexpected EOF")
        };
        match curr.token_type {
            TokenType::While => {
                self.advance();
                let cond = self.expression();
                if !self.matches(vec![TokenType::Colon]) {
                    panic!("Expected colon after while condition")
                }

                if self.matches(vec![TokenType::BeginBlock]) {
                    Expr::While(Box::new(cond), Box::new(self.block()))
                } else {
                    Expr::While(Box::new(cond), Box::new(self.expression()))
                }
            }
            TokenType::If => {
                self.advance();
                let cond = self.expression();

                if !self.matches(vec![TokenType::Colon]) {
                    panic!("Expected colon after if condition: {:?}", self.peek())
                }

                let if_branch = if self.matches(vec![TokenType::BeginBlock]) {
                    self.block()
                } else {
                    self.expression()
                };

                let else_branch = if self.matches(vec![TokenType::Else]) {
                    if self.matches(vec![TokenType::Colon]) {
                        if !self.matches(vec![TokenType::BeginBlock]) {
                            panic!("Expected block after else")
                        }
                        Some(Box::new(self.block()))
                    } else {
                        Some(Box::new(self.expression()))
                    }
                } else {
                    None
                };

                Expr::If(Box::new(cond), Box::new(if_branch), else_branch)
            }
            _ => self.pipeline(),
        }
    }

    fn statement(&mut self) -> Expr {
        self.col = self.peek().expect("Should have token").location.col;
        let mut expr = self.expression();
        match self.peek() {
            Some(Token {
                token_type: TokenType::Colon,
                ..
            }) => {
                if !matches!(expr, Expr::Literal(Value::Identifier(_))) {
                    panic!("Invalid LHS of declaration {expr:?}")
                }
                expr = self.declaration(expr)
            }
            Some(Token {
                token_type: TokenType::Equal,
                ..
            }) => expr = self.assignment(expr),
            _ => (),
        }
        if !self.matches_over_line(TokenType::Semicolon)
            && !self.matches_over_line(TokenType::EndOfFile)
            && !matches!(
                self.previous().token_type,
                TokenType::EndBlock | TokenType::Semicolon
            )
        {
            panic!(
                "Expected semicolon, {:?} {:?}",
                self.previous(),
                self.peek()
            )
        }
        expr
    }

    fn declaration(&mut self, mut expr: Expr) -> Expr {
        if self.matches(vec![TokenType::Colon]) {
            while !matches!(
                self.peek(),
                Some(Token {
                    token_type: TokenType::Equal,
                    ..
                })
            ) {
                if self.peek().is_none() {
                    panic!("Unexpected EOF");
                }
            }
            if self.matches(vec![TokenType::Equal]) {
                expr = Expr::Declaration(Box::new(expr), Some(Box::new(self.expression())))
            } else {
                expr = Expr::Declaration(Box::new(expr), None)
            }
        }
        expr
    }

    fn assignment(&mut self, mut expr: Expr) -> Expr {
        if self.matches(vec![TokenType::Equal]) {
            expr = Expr::Assignment(Box::new(expr), Box::new(self.expression()))
        }
        expr
    }

    fn expression(&mut self) -> Expr {
        self.control_expression()
    }

    fn pipeline(&mut self) -> Expr {
        let mut expr = self.logical_or();
        while self.matches_over_line(TokenType::Pipeline) {
            expr = match self.logical_or() {
                Expr::FunctionCall(e, mut args) => {
                    args.insert(0, expr);
                    Expr::FunctionCall(e, args)
                }
                Expr::Literal(Value::Identifier(name)) => {
                    Expr::FunctionCall(Box::new(Expr::Literal(Value::Identifier(name))), vec![expr])
                }
                _ => panic!("Expected function call in pipeline"),
            }
        }
        expr
    }

    fn logical_or(&mut self) -> Expr {
        let mut expr = self.logical_and();
        while self.matches(vec![TokenType::OrOr])
            || self
                .peek_next()
                .map_or(false, |token| self.col < token.location.col)
                && self.matches_all(vec![TokenType::LineEnd, TokenType::OrOr])
        {
            let op = self.previous();
            let rhs = self.logical_and();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn logical_and(&mut self) -> Expr {
        let mut expr = self.equality();
        while self.matches(vec![TokenType::AndAnd])
            || self
                .peek_next()
                .map_or(false, |token| self.col < token.location.col)
                && self.matches_all(vec![TokenType::LineEnd, TokenType::AndAnd])
        {
            let op = self.previous();
            let rhs = self.equality();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.matches(vec![TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = self.previous();
            let rhs = self.comparison();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.matches(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let rhs = self.term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.matches(vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let rhs = self.factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.matches(vec![
            TokenType::Star,
            TokenType::Slash,
            TokenType::SlashSlash,
            TokenType::Mod,
        ]) {
            let op = self.previous();
            let rhs = self.unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if self.matches(vec![TokenType::Minus, TokenType::Bang]) {
            let op = self.previous();
            let rhs = self.unary();
            return Expr::Unary(op, Box::new(rhs));
        }
        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        if matches!(expr, Expr::Lambda(_, _)) {
            return expr;
        }

        loop {
            if self.matches(vec![TokenType::Dot]) {
                let Token{token_type: TokenType::Identifier(name), ..} = self.advance() else{
                    panic!("Expected name after dot");
                };
                expr = Expr::Get(Box::new(expr), name);
            } else if self.matches(vec![TokenType::Bang]) {
                let mut args = self.arguments();
                if let Expr::Get(lhs, name) = expr {
                    expr = Expr::Literal(Value::Identifier(name));
                    args.insert(0, *lhs);
                }
                expr = Expr::FunctionCall(Box::new(expr), args);
            } else {
                break;
            }
        }
        expr
    }

    fn arguments(&mut self) -> Vec<Expr> {
        let mut args = Vec::default();
        if self.matches(vec![TokenType::BeginBlock]) {
            while !self.matches(vec![TokenType::EndBlock]) {
                args.push(self.logical_or());
                if !self.matches(vec![TokenType::Semicolon])
                    && !matches!(
                        self.previous(),
                        Token {
                            token_type: TokenType::Semicolon,
                            ..
                        }
                    )
                {
                    panic!(
                        "Expected semicolon in function call block: {:?}",
                        self.peek()
                    )
                }
            }
        } else {
            while !self.matches(vec![TokenType::Semicolon])
                && !matches!(
                    self.peek(),
                    Some(Token {
                        token_type: TokenType::Pipeline,
                        ..
                    })
                )
            {
                args.push(self.logical_or());
            }
        }
        args
    }

    fn primary(&mut self) -> Expr {
        if self.matches(vec![TokenType::False]) {
            return Expr::Literal(Value::Bool(false));
        }

        if self.matches(vec![TokenType::True]) {
            return Expr::Literal(Value::Bool(true));
        }

        match self.advance().token_type {
            TokenType::String(value) => Expr::Literal(Value::String(value)),
            TokenType::Identifier(value) => Expr::Literal(Value::Identifier(value)),
            TokenType::Int(value) => Expr::Literal(Value::Int(value)),
            TokenType::LeftParen => {
                let expr = self.expression();
                if !matches!(self.advance().token_type, TokenType::RightParen) {
                    panic!("Unclosed paren");
                }
                Expr::Group(Box::new(expr))
            }
            TokenType::Plus => Expr::Literal(Value::Identifier(String::from("+"))),
            TokenType::Fn => self.function_decl(),
            TokenType::Print => Expr::BuiltinFunction(self.previous()),
            _ => panic!("Unexpected token {:?}", self.previous()),
        }
    }

    fn function_decl(&mut self) -> Expr {
        let mut args = Vec::default();
        while !self.matches(vec![TokenType::ThinArrow]) {
            args.push(self.primary());
        }

        if self.matches(vec![TokenType::BeginBlock]) {
            let Expr::Block(exprs) = self.block() else {
                panic!("Expected block")
            };
            Expr::Lambda(args, exprs)
        } else {
            Expr::Lambda(args, vec![self.expression()])
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Int(i32),
    Bool(bool),
    Identifier(String),
    Lambda(Vec<Expr>, Vec<Expr>, Rc<RefCell<Environment>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(x), Value::Int(other)) => x == other,
            (Value::Bool(x), Value::Bool(other)) => x == other,
            (Value::String(x), Value::String(other)) => x == other,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Value),
    Group(Box<Expr>),
    FunctionCall(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, String),
    Declaration(Box<Expr>, Option<Box<Expr>>),
    Assignment(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    While(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    BuiltinFunction(Token),
    Lambda(Vec<Expr>, Vec<Expr>),
}
