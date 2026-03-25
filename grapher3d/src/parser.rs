#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f32),
    Variable(String),
    Add,
    Sub,
    Multi,
}

pub struct Lexer {
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                // skip whitespace
                ' ' => {
                    chars.next();
                }
                // Operations
                '+' => {
                    tokens.push(Token::Add);
                    chars.next();
                }
                '-' => {
                    tokens.push(Token::Sub);
                    chars.next();
                }
                '*' => {
                    tokens.push(Token::Multi);
                    chars.next();
                }

                // numbers
                '0'..='9' | '.' => {
                    let mut num_str = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num_str.parse().unwrap()));
                }

                // variables
                'a'..='z' => {
                    let mut var_str = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphabetic() {
                            var_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Variable(var_str));
                }
                _ => {
                    chars.next();
                } // Ignore unknown
            }
        }

        Self { tokens }
    }
}

// -------------------
// AST test

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Number(f32),
    Variable(String),
}

impl Expr {
    // turning the tree into WGSL code
    pub fn to_wgsl_code(&self) -> String {
        match self {
            Expr::Number(n) => format!("{:.2}", n),
            Expr::Variable(v) => format!("p.{}", v),
            Expr::Binary(left_expr, op, right_expr) => {
                let l = left_expr.to_wgsl_code();
                let r = right_expr.to_wgsl_code();
                match op {
                    Token::Add => format!("({} + {})", l, r),
                    Token::Sub => format!("({} - {})", l, r),
                    Token::Multi => format!("({} * {})", l, r),
                    _ => unreachable!(),
                }
            }
        }
    }
}

// -------------------
// Parser logic for order of operations

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // get the current position
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    // are tokens done?
    fn has_more_tokens(&self) -> bool {
        self.pos < self.tokens.len()
    }

    // Handle numbers and variables
    pub fn primary_handling(&mut self) -> Box<Expr> {
        match self.current().clone() {
            Token::Number(n) => {
                self.pos += 1;
                Box::new(Expr::Number(n))
            }
            Token::Variable(v) => {
                self.pos += 1;
                Box::new(Expr::Variable(v))
            }
            _ => panic!("unexpected token!"),
        }
    }

    // Handle strong operations
    fn strong_handle(&mut self) -> Box<Expr> {
        let mut left = self.primary_handling();
        while self.has_more_tokens() && matches!(self.current(), Token::Multi) {
            let op = self.current().clone();
            self.pos += 1;
            let right = self.primary_handling();
            left = Box::new(Expr::Binary(left, op, right));
        }
        left
    }

    // Handle weak operations
    pub fn weak_handle(&mut self) -> Box<Expr> {
        let mut left = self.strong_handle();
        while self.has_more_tokens() && matches!(self.current(), Token::Add | Token::Sub) {
            let op = self.current().clone();
            self.pos += 1;
            let right = self.strong_handle();
            left = Box::new(Expr::Binary(left, op, right));
        }
        left
    }
}
