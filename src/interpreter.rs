use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use crate::{
    parser::{Expr, Value},
    scanner::{Token, TokenType},
};

#[derive(Clone, Debug, Default)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn get(&self, identifier: &str) -> Option<Value> {
        self.values.get(identifier).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.borrow().get(identifier))
        })
    }

    pub fn set(&mut self, identifier: &str, value: Value) {
        if !self.values.contains_key(identifier) {
            let parent = self
                .parent
                .as_mut()
                .unwrap_or_else(|| panic!("Error assigning to undefined variable: {identifier:?}"));
            parent.borrow_mut().set(identifier, value);
        } else {
            self.values.insert(identifier.to_string(), value);
        }
    }

    pub fn define(&mut self, identifier: &str, value: Value) {
        self.values.insert(identifier.to_string(), value);
    }
}

pub struct Interpreter<'a, T: Write> {
    environment: Rc<RefCell<Environment>>,
    output: &'a mut T,
}

impl<'a, T: Write> Interpreter<'a, T> {
    pub fn new(output: &'a mut T) -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::default())),
            output,
        }
    }

    pub fn interpret(&mut self, mut exprs: Vec<Expr>) -> Vec<Value> {
        exprs
            .drain(..)
            .map(|expr| self.interpret_expr(&expr))
            .collect()
    }

    pub fn interpret_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Literal(value) => value.clone(),
            Expr::Group(e) => self.interpret_expr(e),
            Expr::Binary(lhs, op, rhs) => self.interpret_binary(lhs, op, rhs),
            Expr::Unary(op, e) => self.interpret_unary(op, e),
            Expr::Declaration(lhs, init) => self.interpret_decl(lhs, init),
            Expr::Assignment(lhs, value) => self.interpret_assignment(lhs, value),
            Expr::While(cond, body) => self.interpret_while(cond, body),
            Expr::Block(exprs) => {
                let new_env = Environment {
                    parent: Some(self.environment.clone()),
                    ..Default::default()
                };
                let old_env = self.environment.clone();
                self.environment = Rc::new(RefCell::new(new_env));
                self.interpret(exprs.clone());
                self.environment = old_env;
                Value::Bool(false)
            }
            Expr::If(cond, true_branch, false_branch) => {
                self.interpret_if(cond, true_branch, false_branch)
            }
            Expr::FunctionCall(id, args) => self.interpret_call(id, args),
            Expr::Lambda(params, body) => {
                Value::Lambda(params.clone(), body.clone(), self.environment.clone())
            }
            Expr::Get(_, _) => todo!(),
            Expr::BuiltinFunction(_) => todo!(),
            Expr::Identifier(identifier) => self
                .environment
                .borrow()
                .get(&identifier)
                .unwrap_or_else(|| panic!("Undefined Variable {identifier:?}"))
                .clone(),
        }
    }

    fn interpret_call(&mut self, id: &Expr, args: &Vec<Expr>) -> Value {
        if let Expr::BuiltinFunction(builtin) = id {
            return self.interpret_builtin(builtin, args);
        }

        let func = self.interpret_expr(id);

        let Value::Lambda(params, body, closure) = func else {
            panic!("Error: Not a function")
        };

        let mut new_env = Environment {
            parent: Some(closure),
            ..Default::default()
        };

        params.iter().zip(args.iter()).for_each(|(param, arg)| {
            let Expr::Identifier(param) = param.clone() else {
                panic!("Invalid function parameter")
            };
            new_env.define(&param, self.interpret_expr(arg))
        });

        let old_env = self.environment.clone();
        self.environment = Rc::new(RefCell::new(new_env));

        let res = self
            .interpret(body)
            .pop()
            .expect("TODO: Functions must have implicit return");

        self.environment = old_env;

        res
    }

    fn interpret_assignment(&mut self, lhs: &Expr, value: &Expr) -> Value {
        let Expr::Identifier(identifier) = lhs else {
            panic!("Invalid LHS of assignment")
        };

        let value = self.interpret_expr(value);
        self.environment.borrow_mut().set(identifier, value.clone());

        value
    }

    fn interpret_builtin(&mut self, token: &Token, args: &Vec<Expr>) -> Value {
        let args = self.interpret(args.clone());

        match token.token_type {
            TokenType::Print => writeln!(self.output, "{args:?}").expect("Failed to write output"),
            _ => panic!("Unknown builtin {token:?}"),
        };
        Value::Bool(false)
    }

    fn interpret_if(
        &mut self,
        cond: &Expr,
        true_branch: &Expr,
        false_branch: &Option<Box<Expr>>,
    ) -> Value {
        let cond = self.interpret_expr(cond);

        if let Value::Bool(true) = cond {
            self.interpret_expr(true_branch)
        } else {
            self.interpret_expr(
                false_branch
                    .as_ref()
                    .expect("TODO: if expression must have else"),
            )
        }
    }

    fn interpret_while(&mut self, cond: &Expr, body: &Expr) -> Value {
        let mut val = self.interpret_expr(cond);
        while let Value::Bool(true) = val {
            self.interpret_expr(body);

            val = self.interpret_expr(cond);
        }
        Value::Bool(false)
    }

    fn interpret_decl(&mut self, lhs: &Expr, init: &Option<Box<Expr>>) -> Value {
        let init = self.interpret_expr(
            init.as_ref()
                .expect("TODO: declarations must have initial value"),
        );

        let Expr::Identifier(identifier) = lhs else {
            panic!("Invalid LHS of declaration")
        };
        self.environment
            .borrow_mut()
            .define(identifier, init.clone());
        init
    }

    fn interpret_unary(&mut self, op: &Token, e: &Expr) -> Value {
        let value = self.interpret_expr(e);

        match (&op.token_type, &value) {
            (TokenType::Minus, Value::Int(x)) => Value::Int(-x),
            (TokenType::Bang, Value::Bool(x)) => Value::Bool(!x),
            _ => panic!("Type error: {op:?} {value:?}"),
        }
    }

    fn interpret_binary(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Value {
        let lhs = self.interpret_expr(lhs);
        let rhs = self.interpret_expr(rhs);

        match (&op.token_type, lhs, rhs) {
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
