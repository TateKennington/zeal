use std::collections::HashMap;

use crate::{
    parser::{Expr, Value},
    scanner::{Token, TokenType},
};

#[derive(Clone, Debug, Default)]
pub struct Environment {
    parent: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn get(&self, identifier: &str) -> Option<&Value> {
        self.values.get(identifier).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.get(identifier))
        })
    }

    pub fn set(&mut self, identifier: String, value: Value) {
        if !self.values.contains_key(&identifier) {
            let parent = self
                .parent
                .as_mut()
                .unwrap_or_else(|| panic!("Error assigning to undefined variable: {identifier:?}"));
            parent.set(identifier, value);
        } else {
            self.values.insert(identifier, value);
        }
    }

    pub fn define(&mut self, identifier: String, value: Value) {
        self.values.insert(identifier, value);
    }

    pub fn push(&mut self) {
        // Self {
        //     parent: Some(Box::new(self)),
        //     values: HashMap::default(),
        // }

        self.parent = Some(Box::new(self.clone()));
        self.values = HashMap::default();
    }

    pub fn pop(&mut self) {
        let parent = std::mem::take(&mut self.parent);
        let parent = parent.expect("Failed to pop scope: No parent");
        self.parent = parent.parent;
        self.values = parent.values;
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::default(),
        }
    }

    pub fn interpret(&mut self, mut exprs: Vec<Expr>) -> Vec<Value> {
        exprs
            .drain(..)
            .map(|expr| {
                let val = self.interpret_expr(expr);
                self.resolve_value(val)
            })
            .collect()
    }

    pub fn resolve_value(&mut self, value: Value) -> Value {
        if let Value::Identifier(identifier) = value {
            self.environment
                .get(&identifier)
                .unwrap_or_else(|| panic!("Undefined Variable {identifier:?}"))
                .clone()
        } else {
            value
        }
    }

    pub fn interpret_expr(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Literal(value) => value,
            Expr::Group(e) => self.interpret_expr(*e),
            Expr::Binary(lhs, op, rhs) => self.interpret_binary(*lhs, op, *rhs),
            Expr::Unary(op, e) => self.interpret_unary(op, *e),
            Expr::Declaration(lhs, init) => self.interpret_decl(*lhs, init.map(|e| *e)),
            Expr::Assignment(lhs, value) => self.interpret_assignment(*lhs, *value),
            Expr::While(cond, body) => self.interpret_while(*cond, *body),
            Expr::Block(exprs) => {
                self.environment.push();
                self.interpret(exprs);
                self.environment.pop();
                Value::Bool(false)
            }
            Expr::If(cond, true_branch, false_branch) => {
                self.interpret_if(*cond, *true_branch, false_branch.map(|e| *e))
            }
            Expr::BuiltinFunction(token, args) => self.interpret_builtin(token, args),
            e => todo!("{e:?}"),
        }
    }

    fn interpret_assignment(&mut self, lhs: Expr, value: Expr) -> Value {
        let Value::Identifier(identifier) = self.interpret_expr(lhs) else {
            panic!("Invalid LHS of assignment")
        };

        let value = self.interpret_expr(value);
        let value = self.resolve_value(value);

        self.environment.set(identifier, value.clone());

        value
    }

    fn interpret_builtin(&mut self, token: Token, args: Vec<Expr>) -> Value {
        let args = self.interpret(args);
        let args: Vec<Value> = args
            .iter()
            .map(|arg| self.resolve_value(arg.clone()))
            .collect();

        match token.token_type {
            TokenType::Print => println!("{args:?}"),
            _ => panic!("Unknown builtin {token:?}"),
        };
        Value::Bool(false)
    }

    fn interpret_if(&mut self, cond: Expr, true_branch: Expr, false_branch: Option<Expr>) -> Value {
        let cond = self.interpret_expr(cond);
        let cond = self.resolve_value(cond);

        if let Value::Bool(true) = cond {
            self.interpret_expr(true_branch)
        } else {
            self.interpret_expr(false_branch.expect("TODO: if expression must have else"))
        }
    }

    fn interpret_while(&mut self, cond: Expr, body: Expr) -> Value {
        let val = self.interpret_expr(cond.clone());
        let mut cond_val = self.resolve_value(val);
        while let Value::Bool(true) = cond_val {
            self.interpret_expr(body.clone());

            let val = self.interpret_expr(cond.clone());
            cond_val = self.resolve_value(val);
        }
        Value::Bool(false)
    }

    fn interpret_decl(&mut self, lhs: Expr, init: Option<Expr>) -> Value {
        let init = self.interpret_expr(init.expect("TODO: declarations must have initial value"));
        let init = self.resolve_value(init);

        let Value::Identifier(identifier) = self.interpret_expr(lhs) else {
            panic!("Invalid LHS of declaration")
        };
        self.environment.define(identifier, init.clone());
        init
    }

    fn interpret_unary(&mut self, op: Token, e: Expr) -> Value {
        let value = self.interpret_expr(e);
        let value = self.resolve_value(value);

        match (op.token_type, value) {
            (TokenType::Minus, Value::Int(x)) => Value::Int(-x),
            (TokenType::Bang, Value::Bool(x)) => Value::Bool(!x),
            _ => panic!("Type error"),
        }
    }

    fn interpret_binary(&mut self, lhs: Expr, op: Token, rhs: Expr) -> Value {
        let lhs = self.interpret_expr(lhs);
        let lhs = self.resolve_value(lhs);
        let rhs = self.interpret_expr(rhs);
        let rhs = self.resolve_value(rhs);

        match (op.token_type, lhs, rhs) {
            (TokenType::Minus, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs - rhs),
            (TokenType::Plus, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs + rhs),
            (TokenType::Star, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs * rhs),
            (TokenType::Mod, Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs % rhs),
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

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
