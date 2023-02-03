use crate::{
    parser::{Expr, Value},
    scanner::{Token, TokenType},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, mut exprs: Vec<Expr>) -> Vec<Value> {
        exprs.drain(..).map(|expr| expr.interpret()).collect()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Expr {
    pub fn interpret(self) -> Value {
        match self {
            Expr::Literal(value) => value,
            Expr::Group(e) => e.interpret(),
            Expr::Binary(lhs, op, rhs) => Expr::interpret_binary(*lhs, op, *rhs),
            Expr::Unary(op, e) => Expr::interpret_unary(op, *e),
            e => todo!("{e:?}"),
        }
    }

    fn interpret_unary(op: Token, e: Expr) -> Value {
        let value = e.interpret();
        match (op.token_type, value) {
            (TokenType::Minus, Value::Int(x)) => Value::Int(-x),
            (TokenType::Bang, Value::Bool(x)) => Value::Bool(!x),
            _ => panic!("Type error"),
        }
    }

    fn interpret_binary(lhs: Expr, op: Token, rhs: Expr) -> Value {
        let lhs = lhs.interpret();
        let rhs = rhs.interpret();
        match (op.token_type, lhs, rhs) {
            (TokenType::Minus, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs - rhs),
            (TokenType::Plus, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs + rhs),
            (TokenType::Star, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs * rhs),
            (TokenType::SlashSlash, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs / rhs),
            (TokenType::AndAnd, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs && rhs),
            (TokenType::OrOr, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs || rhs),
            (TokenType::Greater, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs > rhs),
            (TokenType::GreaterEqual, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs >= rhs),
            (TokenType::Less, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs < rhs),
            (TokenType::LessEqual, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs <= rhs),
            (TokenType::EqualEqual, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs == rhs),
            (TokenType::EqualEqual, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs == rhs),
            (TokenType::EqualEqual, Value::String(lhs), Value::String(rhs)) => {
                Value::Bool(lhs == rhs)
            }
            (TokenType::BangEqual, Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs != rhs),
            (TokenType::BangEqual, Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs != rhs),
            (TokenType::BangEqual, Value::String(lhs), Value::String(rhs)) => {
                Value::Bool(lhs != rhs)
            }
            _ => panic!("Type error"),
        }
    }
}
