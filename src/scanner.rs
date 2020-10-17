use lazy_static::lazy_static;

use std::collections::HashMap;

use crate::{
    object::LoxObject,
    object::Object,
    token::{Token, TokenKind},
};

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenKind> = {
        let mut keywords = HashMap::new();

        keywords.insert(String::from("and"), TokenKind::And);
        keywords.insert(String::from("class"), TokenKind::Class);
        keywords.insert(String::from("else"), TokenKind::Else);
        keywords.insert(String::from("false"), TokenKind::False);
        keywords.insert(String::from("for"), TokenKind::For);
        keywords.insert(String::from("fun"), TokenKind::Fun);
        keywords.insert(String::from("if"), TokenKind::If);
        keywords.insert(String::from("nil"), TokenKind::Nil);
        keywords.insert(String::from("or"), TokenKind::Or);
        keywords.insert(String::from("print"), TokenKind::Print);
        keywords.insert(String::from("return"), TokenKind::Return);
        keywords.insert(String::from("super"), TokenKind::Super);
        keywords.insert(String::from("this"), TokenKind::This);
        keywords.insert(String::from("true"), TokenKind::True);
        keywords.insert(String::from("var"), TokenKind::Var);
        keywords.insert(String::from("while"), TokenKind::While);

        keywords
    };
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],

            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            String::from(""),
            Object::nil(),
            self.line,
        ));

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenKind::LParen, Object::nil()),
            ')' => self.add_token(TokenKind::RParen, Object::nil()),
            '{' => self.add_token(TokenKind::LBrace, Object::nil()),
            '}' => self.add_token(TokenKind::RBrace, Object::nil()),
            ',' => self.add_token(TokenKind::Comma, Object::nil()),
            '.' => self.add_token(TokenKind::Dot, Object::nil()),
            '-' => self.add_token(TokenKind::Minus, Object::nil()),
            '+' => self.add_token(TokenKind::Plus, Object::nil()),
            ';' => self.add_token(TokenKind::Semicolon, Object::nil()),
            '*' => self.add_token(TokenKind::Star, Object::nil()),
            '!' => {
                if self.matches('=') {
                    self.add_token(TokenKind::BangEqual, Object::nil());
                } else {
                    self.add_token(TokenKind::Bang, Object::nil());
                }
            }
            '=' => {
                if self.matches('=') {
                    self.add_token(TokenKind::EqualEqual, Object::nil());
                } else {
                    self.add_token(TokenKind::Equal, Object::nil());
                }
            }
            '<' => {
                if self.matches('=') {
                    self.add_token(TokenKind::LessEqual, Object::nil());
                } else {
                    self.add_token(TokenKind::Less, Object::nil());
                }
            }
            '>' => {
                if self.matches('=') {
                    self.add_token(TokenKind::GreaterEqual, Object::nil());
                } else {
                    self.add_token(TokenKind::Greater, Object::nil());
                }
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash, Object::nil());
                }
            }
            c if c.is_whitespace() => {
                if c == '\n' {
                    self.line += 1;
                }
            }
            '"' => self.string(),
            c if c.is_digit(10) => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            _ => crate::error(self.line, "Unexpected character."),
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let kind = KEYWORDS
            .get(&text)
            .cloned()
            .unwrap_or(TokenKind::Identifier);
        self.add_token(kind, Object::nil());
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .unwrap();
        self.add_token(TokenKind::Number, Object::new_number(value));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.at_end() {
            crate::error(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect::<String>();
        self.add_token(TokenKind::String, Object::new_string(value));
    }

    fn add_token(&mut self, kind: TokenKind, literal: LoxObject) {
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        self.tokens.push(Token::new(kind, text, literal, self.line));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn peek(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
