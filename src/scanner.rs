pub struct Scanner {
    stream: String,
    curr_loc: Location,
    tokens: Vec<Token>,
    open_block: Option<Location>,
    block_levels: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Plus,
    Semicolon,
    Colon,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    AndAnd,
    Or,
    OrOr,
    Slash,
    SlashSlash,
    Mod,
    ModMod,
    Minus,
    ThinArrow,
    Pipeline,

    // Literals.
    Identifier(String),
    String(String),
    Int(i32),

    // Keywords.
    Then,
    Else,
    False,
    Fn,
    For,
    While,
    If,
    Print,
    Return,
    True,

    EndOfFile,

    Comment(String),

    BeginBlock,
    EndBlock,
}

#[derive(Clone, Copy, Debug)]
struct Location {
    line: usize,
    col: usize,
    index: usize,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    location: Location,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            stream: String::default(),
            curr_loc: Location {
                col: 0,
                index: 0,
                line: 0,
            },
            tokens: vec![],
            open_block: None,
            block_levels: Vec::default(),
        }
    }

    pub fn emit_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token {
            token_type,
            location: self.curr_loc,
        })
    }

    pub fn check(&mut self, lexeme: char) -> bool {
        if self.curr_loc.index >= self.stream.len() {
            false
        } else {
            let c = self.stream.chars().nth(self.curr_loc.index);
            if let Some(c) = c {
                self.next();
                return c == lexeme;
            }
            false
        }
    }

    fn emit_open_block(&mut self) {
        if let Some(curr_block_col) = self.block_levels.last() {
            if curr_block_col >= &self.curr_loc.col {
                todo!("Handle error here");
            }
        }
        self.open_block = None;
        self.block_levels.push(self.curr_loc.col);
        self.emit_token(TokenType::BeginBlock);
    }

    fn emit_closed_blocks(&mut self) {
        while let Some(level) = self.block_levels.last() {
            if level <= &self.curr_loc.col {
                break;
            }
            self.block_levels.pop();
            self.emit_token(TokenType::EndBlock);
        }
    }

    fn emit_string(&mut self, boundary: char) {
        let mut value = String::default();
        loop {
            match self.next() {
                None => todo!("Handle error here"),
                Some(c) => {
                    if c == boundary {
                        self.emit_token(TokenType::String(value));
                        return;
                    }
                    value.push(c);
                }
            }
        }
    }

    fn emit_int(&mut self, first: char) {
        let mut value = String::from(first);
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.next();
            value.push(c);
        }
        let value = value.parse().expect("Failed to parse int");
        self.emit_token(TokenType::Int(value))
    }

    fn identifier_symbol(c: char) -> bool {
        let allowed_symbols = ['-', '>', '<', '+', '/', '%', '&', '|', '!', '=', '*'];

        if allowed_symbols.contains(&c) {
            return true;
        }

        false
    }

    fn identifier_character(c: char) -> bool {
        if c.is_alphanumeric() || c == '_' {
            return true;
        }

        false
    }

    fn scan_identifier(&mut self, first: char) -> String {
        let mut value = String::from(first);
        while let Some(c) = self.peek() {
            if (Scanner::identifier_character(first) && !Scanner::identifier_character(c))
                || (Scanner::identifier_symbol(first) && !Scanner::identifier_symbol(c))
            {
                break;
            }
            self.next();
            value.push(c);
        }
        value
    }

    fn emit_end_of_file(&mut self) {
        if !self.curr_loc.col == 0 {
            self.curr_loc.col = 0;
            self.curr_loc.line += 1;
        }
        self.emit_closed_blocks();
    }

    pub fn scan(&mut self, line: String) -> Vec<Token> {
        self.stream = line;
        while let Some(c) = self.next() {
            if !matches!(c, '\n' | ' ' | '\t' | '\r') {
                if let Some(opening_loc) = self.open_block {
                    if self.curr_loc.line == opening_loc.line {
                        self.open_block = None;
                    } else {
                        self.emit_open_block();
                    }
                }

                if !self.block_levels.is_empty() {
                    self.emit_closed_blocks();
                }
            }

            match c {
                //One character tokens
                '(' => self.emit_token(TokenType::LeftParen),
                ')' => self.emit_token(TokenType::RightParen),
                '{' => self.emit_token(TokenType::RightBrace),
                '}' => self.emit_token(TokenType::LeftBrace),
                ',' => self.emit_token(TokenType::Comma),
                '.' => self.emit_token(TokenType::Dot),
                ';' => self.emit_token(TokenType::Semicolon),
                ':' => {
                    self.open_block = Some(self.curr_loc);
                    self.emit_token(TokenType::Colon)
                }
                //ignored characters
                '\n' | ' ' | '\t' | '\r' => {}
                '"' => self.emit_string('"'),
                '\'' => self.emit_string('\''),
                c if c.is_ascii_digit() => {
                    self.emit_int(c);
                }
                c => {
                    let id = self.scan_identifier(c);
                    match id.as_str() {
                        //One or two character tokens
                        "!=" => self.emit_token(TokenType::BangEqual),
                        "!" => self.emit_token(TokenType::Bang),
                        "=" => self.emit_token(TokenType::Equal),
                        "==" => self.emit_token(TokenType::EqualEqual),
                        "<=" => self.emit_token(TokenType::LessEqual),
                        "<" => self.emit_token(TokenType::Less),
                        ">" => self.emit_token(TokenType::Greater),
                        ">=" => self.emit_token(TokenType::GreaterEqual),
                        "&" => self.emit_token(TokenType::And),
                        "&&" => self.emit_token(TokenType::AndAnd),
                        "|" => self.emit_token(TokenType::Or),
                        "||" => self.emit_token(TokenType::OrOr),
                        "/" => self.emit_token(TokenType::Slash),
                        "//" => self.emit_token(TokenType::SlashSlash),
                        "%" => self.emit_token(TokenType::Mod),
                        "%%" => self.emit_token(TokenType::ModMod),
                        "-" => self.emit_token(TokenType::Minus),
                        "+" => self.emit_token(TokenType::Plus),
                        "*" => self.emit_token(TokenType::Star),
                        "|>" => self.emit_token(TokenType::Pipeline),
                        "->" => {
                            self.open_block = Some(self.curr_loc);
                            self.emit_token(TokenType::ThinArrow)
                        }
                        "false" => self.emit_token(TokenType::False),
                        "true" => self.emit_token(TokenType::True),
                        "fn" => self.emit_token(TokenType::Fn),
                        "for" => self.emit_token(TokenType::For),
                        "while" => self.emit_token(TokenType::While),
                        "return" => self.emit_token(TokenType::Return),
                        "print" => self.emit_token(TokenType::Print),
                        "if" => self.emit_token(TokenType::If),
                        "then" => self.emit_token(TokenType::Then),
                        "else" => self.emit_token(TokenType::Else),
                        _ => self.emit_token(TokenType::Identifier(id)),
                    }
                }
            }
        }
        self.emit_end_of_file();
        self.tokens.drain(..).collect()
    }

    fn peek(&mut self) -> Option<char> {
        if self.curr_loc.index >= self.stream.len() {
            None
        } else {
            self.stream.chars().nth(self.curr_loc.index)
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.curr_loc.index >= self.stream.len() {
            None
        } else {
            let c = self.stream.chars().nth(self.curr_loc.index);
            match c {
                Some('\n') => {
                    self.curr_loc.line += 1;
                    self.curr_loc.index += 1;
                    self.curr_loc.col = 0;
                }
                Some('\t') => {
                    self.curr_loc.index += 1;
                    self.curr_loc.col += 4;
                }
                Some(_) => {
                    self.curr_loc.col += 1;
                    self.curr_loc.index += 1;
                }
                None => {}
            }
            c
        }
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}
