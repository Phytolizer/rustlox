pub struct Scanner<'source> {
    pub source: &'source [u8],
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
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

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Vec<u8>,
    pub line: usize,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        match self.advance() {
            b'(' => return self.make_token(TokenKind::LeftParen),
            b')' => return self.make_token(TokenKind::RightParen),
            b'{' => return self.make_token(TokenKind::LeftBrace),
            b'}' => return self.make_token(TokenKind::RightBrace),
            b';' => return self.make_token(TokenKind::Semicolon),
            b',' => return self.make_token(TokenKind::Comma),
            b'.' => return self.make_token(TokenKind::Dot),
            b'-' => return self.make_token(TokenKind::Minus),
            b'+' => return self.make_token(TokenKind::Plus),
            b'/' => return self.make_token(TokenKind::Slash),
            b'*' => return self.make_token(TokenKind::Star),
            b'!' => {
                if self.matches(b'=') {
                    return self.make_token(TokenKind::BangEqual);
                } else {
                    return self.make_token(TokenKind::Bang);
                }
            }
            b'=' => {
                if self.matches(b'=') {
                    return self.make_token(TokenKind::EqualEqual);
                } else {
                    return self.make_token(TokenKind::Equal);
                }
            }
            b'<' => {
                if self.matches(b'=') {
                    return self.make_token(TokenKind::LessEqual);
                } else {
                    return self.make_token(TokenKind::Less);
                }
            }
            b'>' => {
                if self.matches(b'=') {
                    return self.make_token(TokenKind::GreaterEqual);
                } else {
                    return self.make_token(TokenKind::Greater);
                }
            }
            b'"' => return self.string(),
            c if c.is_ascii_digit() => return self.number(),
            c if Self::is_alpha(c) => return self.identifier(),
            _ => {}
        }

        self.error_token("Unexpected character.")
    }

    fn is_alpha(c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_'
    }

    fn identifier(&mut self) -> Token {
        while Self::is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }

        self.make_token(self.identifier_kind())
    }

    fn identifier_kind(&self) -> TokenKind {
        TokenKind::Identifier
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error_token("Unterminated string.")
        } else {
            self.advance();
            self.make_token(TokenKind::String)
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.source[self.start..self.current].to_owned(),
            line: self.line,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            kind: TokenKind::Error,
            lexeme: message.as_bytes().to_owned(),
            line: self.line,
        }
    }
}
