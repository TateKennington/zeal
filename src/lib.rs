use std::{fs::read_to_string, io, path::Path};

use interpreter::Interpreter;
use parser::{Expr, Parser, Value};
use scanner::{Scanner, Token};

mod interpreter;
mod parser;
mod scanner;

pub struct Compiler {
    scanner: Scanner,
    parser: Parser,
    interpreter: Interpreter,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            scanner: Scanner::default(),
            parser: Parser::default(),
            interpreter: Interpreter::default(),
        }
    }

    pub fn run(&mut self, path: &Path) -> io::Result<()> {
        let contents = read_to_string(path)?;
        let _token_stream = self.scanner.scan(contents);
        Ok(())
    }

    pub fn run_line(&mut self, line: &str) {
        let _token_stream = self.scanner.scan(String::from(line));
    }

    pub fn scan_line(&mut self, line: &str) -> Vec<scanner::Token> {
        self.scanner.scan(String::from(line))
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Expr> {
        self.parser.parse(tokens)
    }

    pub fn evaluate(&mut self, expressions: Vec<Expr>) -> Vec<Value> {
        self.interpreter.interpret(expressions)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
