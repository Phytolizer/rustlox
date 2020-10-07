use std::{convert::TryFrom, io::Write};

use crate::{
    chunk::Chunk,
    chunk::OpCode,
    common::DEBUG_PRINT_CODE,
    debug::disassemble_chunk,
    object::Obj,
    scanner::{Scanner, Token, TokenKind},
    value::Value,
};

#[repr(usize)]
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, num_enum::TryFromPrimitive)]
enum Precedence {
    None = 0,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

type ParseFn<'source, 'chunk> = fn(&mut Parser<'source, 'chunk>) -> eyre::Result<()>;

struct ParseRule<'source, 'chunk> {
    prefix: Option<ParseFn<'source, 'chunk>>,
    infix: Option<ParseFn<'source, 'chunk>>,
    precedence: Precedence,
}

fn get_rule<'source, 'chunk>(kind: TokenKind) -> ParseRule<'source, 'chunk> {
    match kind {
        TokenKind::LeftParen => ParseRule {
            prefix: Some(Parser::grouping),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::RightParen => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::LeftBrace => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::RightBrace => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Comma => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Dot => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Minus => ParseRule {
            prefix: Some(Parser::unary),
            infix: Some(Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Plus => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Semicolon => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Slash => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Star => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Bang => ParseRule {
            prefix: Some(Parser::unary),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::BangEqual => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Equality,
        },
        TokenKind::Equal => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Equality,
        },
        TokenKind::Less => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::LessEqual => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Greater => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Identifier => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::String => ParseRule {
            prefix: Some(Parser::string),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Number => ParseRule {
            prefix: Some(Parser::number),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::And => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Class => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Else => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::False => ParseRule {
            prefix: Some(Parser::literal),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::For => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Fun => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::If => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Nil => ParseRule {
            prefix: Some(Parser::literal),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Or => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Print => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Return => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Super => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::This => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::True => ParseRule {
            prefix: Some(Parser::literal),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Var => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::While => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Error => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Eof => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    }
}

struct Parser<'source, 'chunk> {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    scanner: Scanner<'source>,
    current_chunk: &'chunk mut Chunk,
}

impl<'source, 'chunk> Parser<'source, 'chunk> {
    fn new(scanner: Scanner<'source>, chunk: &'chunk mut Chunk) -> Self {
        Self {
            current: Token {
                kind: TokenKind::Eof,
                lexeme: vec![],
                line: 0,
            },
            previous: Token {
                kind: TokenKind::Eof,
                lexeme: vec![],
                line: 0,
            },
            had_error: false,
            panic_mode: false,
            scanner,
            current_chunk: chunk,
        }
    }

    fn advance(&mut self) -> eyre::Result<()> {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            if self.current.kind != TokenKind::Error {
                break;
            }

            self.error_at_current(&self.current.lexeme.clone())?;
        }
        Ok(())
    }

    fn expression(&mut self) -> eyre::Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn expression_statement(&mut self) -> eyre::Result<()> {
        self.expression()?;
        self.consume(TokenKind::Semicolon, b"Expect ';' after expression.")?;
        self.emit_byte(OpCode::Pop);
        Ok(())
    }

    fn print_statement(&mut self) -> eyre::Result<()> {
        self.expression()?;
        self.consume(TokenKind::Semicolon, b"Expect ';' after value.")?;
        self.emit_byte(OpCode::Print);
        Ok(())
    }

    fn synchronize(&mut self) -> eyre::Result<()> {
        self.panic_mode = false;

        while self.current.kind != TokenKind::Eof {
            if self.previous.kind == TokenKind::Semicolon {
                return Ok(());
            }

            match self.current.kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => {
                    return Ok(());
                }
                _ => {}
            }

            self.advance()?;
        }
        Ok(())
    }

    fn declaration(&mut self) -> eyre::Result<()> {
        self.statement()?;

        if self.panic_mode {
            self.synchronize();
        }

        Ok(())
    }

    fn statement(&mut self) -> eyre::Result<()> {
        if self.matches(TokenKind::Print)? {
            self.print_statement()?;
        } else {
            self.expression_statement()?;
        }
        Ok(())
    }

    fn consume(&mut self, expected: TokenKind, message: &[u8]) -> eyre::Result<()> {
        if self.current.kind == expected {
            self.advance()?;
            return Ok(());
        }

        self.error_at_current(message)?;
        Ok(())
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    fn matches(&mut self, kind: TokenKind) -> eyre::Result<bool> {
        if !self.check(kind) {
            Ok(false)
        } else {
            self.advance()?;
            Ok(true)
        }
    }

    fn error_at_current(&mut self, message: &[u8]) -> eyre::Result<()> {
        self.error_at(&self.current.clone(), message)?;
        Ok(())
    }

    fn error(&mut self, message: &[u8]) -> eyre::Result<()> {
        self.error_at(&self.previous.clone(), message)?;
        Ok(())
    }

    fn error_at(&mut self, token: &Token, message: &[u8]) -> eyre::Result<()> {
        if self.panic_mode {
            return Ok(());
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        if token.kind == TokenKind::Eof {
            eprint!(" at end");
        } else if token.kind != TokenKind::Error {
            eprint!(" at '");
            std::io::stderr().write_all(&token.lexeme)?;
            eprint!("'");
        }

        eprint!(": ");
        std::io::stderr().write_all(message)?;
        eprintln!();

        self.had_error = true;
        Ok(())
    }

    fn emit_byte<B: Into<u8>>(&mut self, byte: B) {
        self.current_chunk.write(byte, self.previous.line);
    }

    fn emit_bytes<B1: Into<u8>, B2: Into<u8>>(&mut self, byte1: B1, byte2: B2) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn make_constant(&mut self, value: Value) -> eyre::Result<u8> {
        let constant = self.current_chunk.add_constant(value);
        if constant > std::u8::MAX as usize {
            self.error(b"Too many constants in one chunk.")?;
            Ok(0)
        } else {
            Ok(constant as u8)
        }
    }

    fn emit_constant(&mut self, constant: Value) -> eyre::Result<()> {
        let offset = self.make_constant(constant)?;
        self.emit_bytes(OpCode::Constant, offset);
        Ok(())
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if DEBUG_PRINT_CODE && !self.had_error {
            disassemble_chunk(&self.current_chunk, "code");
        }
    }

    fn binary(&mut self) -> eyre::Result<()> {
        let operator_kind = self.previous.kind;

        let rule = get_rule(operator_kind);
        self.parse_precedence(Precedence::try_from(rule.precedence as usize + 1).unwrap())?;

        match operator_kind {
            TokenKind::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenKind::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenKind::Greater => self.emit_byte(OpCode::Greater),
            TokenKind::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenKind::Less => self.emit_byte(OpCode::Less),
            TokenKind::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            TokenKind::Plus => self.emit_byte(OpCode::Add),
            TokenKind::Minus => self.emit_byte(OpCode::Sub),
            TokenKind::Star => self.emit_byte(OpCode::Mul),
            TokenKind::Slash => self.emit_byte(OpCode::Div),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn literal(&mut self) -> eyre::Result<()> {
        match self.previous.kind {
            TokenKind::False => self.emit_byte(OpCode::False),
            TokenKind::Nil => self.emit_byte(OpCode::Nil),
            TokenKind::True => self.emit_byte(OpCode::True),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn grouping(&mut self) -> eyre::Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, b"Expect ')' after expression.")?;
        Ok(())
    }

    fn number(&mut self) -> eyre::Result<()> {
        let value = String::from_utf8_lossy(&self.previous.lexeme).parse::<f64>()?;
        self.emit_constant(Value::Number(value))?;
        Ok(())
    }

    fn string(&mut self) -> eyre::Result<()> {
        self.emit_constant(Value::Obj(Box::new(Obj::String(
            self.previous.lexeme[1..self.previous.lexeme.len() - 1].to_owned(),
        ))))?;
        Ok(())
    }

    fn unary(&mut self) -> eyre::Result<()> {
        let operator_kind = self.previous.kind;

        self.parse_precedence(Precedence::Unary)?;

        match operator_kind {
            TokenKind::Minus => self.emit_byte(OpCode::Negate),
            TokenKind::Bang => self.emit_byte(OpCode::Not),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> eyre::Result<()> {
        self.advance()?;
        let prefix_rule = get_rule(self.previous.kind).prefix;
        match prefix_rule {
            None => {
                self.error(b"Expect expression.")?;
            }
            Some(prefix_rule) => prefix_rule(self)?,
        }

        while precedence <= get_rule(self.current.kind).precedence {
            self.advance()?;
            if let Some(infix_rule) = get_rule(self.previous.kind).infix {
                infix_rule(self)?;
            }
        }
        Ok(())
    }
}

pub fn compile(source: &[u8], chunk: &mut crate::chunk::Chunk) -> eyre::Result<bool> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner, chunk);
    parser.advance()?;

    while !parser.matches(TokenKind::Eof)? {
        parser.declaration()?;
    }

    parser.end_compiler();
    Ok(!parser.had_error)
}
