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

type ParseFn<'source, 'chunk> = fn(&mut Compiler<'source, 'chunk>, bool) -> eyre::Result<()>;

struct ParseRule<'source, 'chunk> {
    prefix: Option<ParseFn<'source, 'chunk>>,
    infix: Option<ParseFn<'source, 'chunk>>,
    precedence: Precedence,
}

fn get_rule<'source, 'chunk>(kind: TokenKind) -> ParseRule<'source, 'chunk> {
    match kind {
        TokenKind::LeftParen => ParseRule {
            prefix: Some(Compiler::grouping),
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
            prefix: Some(Compiler::unary),
            infix: Some(Compiler::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Plus => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Semicolon => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Slash => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Star => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Bang => ParseRule {
            prefix: Some(Compiler::unary),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::BangEqual => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Equality,
        },
        TokenKind::Equal => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Equality,
        },
        TokenKind::Less => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::LessEqual => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Greater => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(Compiler::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Identifier => ParseRule {
            prefix: Some(Compiler::variable),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::String => ParseRule {
            prefix: Some(Compiler::string),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Number => ParseRule {
            prefix: Some(Compiler::number),
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
            prefix: Some(Compiler::literal),
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
            prefix: Some(Compiler::literal),
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
            prefix: Some(Compiler::literal),
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
}

impl<'source, 'chunk> Compiler<'source, 'chunk> {
    fn advance(&mut self) -> eyre::Result<()> {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = self.parser.scanner.scan_token();
            if self.parser.current.kind != TokenKind::Error {
                break;
            }

            self.error_at_current(&self.parser.current.lexeme.clone())?;
        }
        Ok(())
    }

    fn expression(&mut self) -> eyre::Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn block(&mut self) -> eyre::Result<()> {
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            self.declaration()?;
        }

        self.consume(TokenKind::RightBrace, b"Expect '}' after block.")?;
        Ok(())
    }

    fn var_declaration(&mut self) -> eyre::Result<()> {
        let global = self.parse_variable(b"Expect variable name.")?;

        if self.matches(TokenKind::Equal)? {
            self.expression()?;
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenKind::Semicolon,
            b"Expect ';' after variable declaration.",
        )?;

        self.define_variable(global);
        Ok(())
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
        self.parser.panic_mode = false;

        while self.parser.current.kind != TokenKind::Eof {
            if self.parser.previous.kind == TokenKind::Semicolon {
                return Ok(());
            }

            match self.parser.current.kind {
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
        if self.matches(TokenKind::Var)? {
            self.var_declaration()?;
        } else {
            self.statement()?;
        }

        if self.parser.panic_mode {
            self.synchronize()?;
        }

        Ok(())
    }

    fn statement(&mut self) -> eyre::Result<()> {
        if self.matches(TokenKind::Print)? {
            self.print_statement()?;
        } else if self.matches(TokenKind::LeftBrace)? {
            self.begin_scope();
            self.block()?;
            self.end_scope();
        } else {
            self.expression_statement()?;
        }
        Ok(())
    }

    fn consume(&mut self, expected: TokenKind, message: &[u8]) -> eyre::Result<()> {
        if self.parser.current.kind == expected {
            self.advance()?;
            return Ok(());
        }

        self.error_at_current(message)?;
        Ok(())
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        self.parser.current.kind == kind
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
        self.error_at(&self.parser.current.clone(), message)?;
        Ok(())
    }

    fn error(&mut self, message: &[u8]) -> eyre::Result<()> {
        self.error_at(&self.parser.previous.clone(), message)?;
        Ok(())
    }

    fn error_at(&mut self, token: &Token, message: &[u8]) -> eyre::Result<()> {
        if self.parser.panic_mode {
            return Ok(());
        }
        self.parser.panic_mode = true;
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

        self.parser.had_error = true;
        Ok(())
    }

    fn emit_byte<B: Into<u8>>(&mut self, byte: B) {
        self.parser
            .current_chunk
            .write(byte, self.parser.previous.line);
    }

    fn emit_bytes<B1: Into<u8>, B2: Into<u8>>(&mut self, byte1: B1, byte2: B2) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn make_constant(&mut self, value: Value) -> eyre::Result<u8> {
        let constant = self.parser.current_chunk.add_constant(value);
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
        if DEBUG_PRINT_CODE && !self.parser.had_error {
            disassemble_chunk(&self.parser.current_chunk, "code");
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self.local_count > 0
            && self.locals[self.local_count - 1].depth > self.scope_depth as isize
        {
            self.emit_byte(OpCode::Pop);
            self.local_count -= 1;
        }
    }

    fn binary(&mut self, _can_assign: bool) -> eyre::Result<()> {
        let operator_kind = self.parser.previous.kind;

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

    fn literal(&mut self, _can_assign: bool) -> eyre::Result<()> {
        match self.parser.previous.kind {
            TokenKind::False => self.emit_byte(OpCode::False),
            TokenKind::Nil => self.emit_byte(OpCode::Nil),
            TokenKind::True => self.emit_byte(OpCode::True),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn grouping(&mut self, _can_assign: bool) -> eyre::Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, b"Expect ')' after expression.")?;
        Ok(())
    }

    fn number(&mut self, _can_assign: bool) -> eyre::Result<()> {
        let value = String::from_utf8_lossy(&self.parser.previous.lexeme).parse::<f64>()?;
        self.emit_constant(Value::Number(value))?;
        Ok(())
    }

    fn string(&mut self, _can_assign: bool) -> eyre::Result<()> {
        self.emit_constant(Value::Obj(Box::new(Obj::String(
            self.parser.previous.lexeme[1..self.parser.previous.lexeme.len() - 1].to_owned(),
        ))))?;
        Ok(())
    }

    fn resolve_local(&self, name: &Token) -> isize {
        for i in 0..self.local_count {
            let local = &self.locals[i];
            if name.lexeme == local.name.lexeme {
                return i as isize;
            }
        }
        -1
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) -> eyre::Result<()> {
        let mut arg = self.resolve_local(name);
        let get_op: OpCode;
        let set_op: OpCode;
        if arg != -1 {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else {
            arg = self.identifier_constant(name)? as isize;
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        if can_assign && self.matches(TokenKind::Equal)? {
            self.expression()?;
            self.emit_bytes(set_op, arg as u8);
        } else {
            self.emit_bytes(get_op, arg as u8);
        }
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> eyre::Result<()> {
        self.named_variable(&self.parser.previous.clone(), can_assign)
    }

    fn unary(&mut self, _can_assign: bool) -> eyre::Result<()> {
        let operator_kind = self.parser.previous.kind;

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
        let prefix_rule = get_rule(self.parser.previous.kind).prefix;
        let can_assign = precedence <= Precedence::Assignment;
        match prefix_rule {
            None => {
                self.error(b"Expect expression.")?;
            }
            Some(prefix_rule) => prefix_rule(self, can_assign)?,
        }

        while precedence <= get_rule(self.parser.current.kind).precedence {
            self.advance()?;
            if let Some(infix_rule) = get_rule(self.parser.previous.kind).infix {
                infix_rule(self, can_assign)?;
            }
        }

        if can_assign && self.matches(TokenKind::Equal)? {
            self.error(b"Invalid assignment target.")?;
        }
        Ok(())
    }

    fn identifier_constant(&mut self, name: &Token) -> eyre::Result<u8> {
        self.make_constant(Value::Obj(Box::new(Obj::String(name.lexeme.clone()))))
    }

    fn add_local(&mut self, name: Token) -> eyre::Result<()> {
        if self.locals.len() == std::u8::MAX as usize + 1 {
            self.error(b"Too many local variables in function.")?;
            return Ok(());
        }

        let local = Local {
            name,
            depth: self.scope_depth as isize,
        };
        self.locals.push(local);
        self.local_count += 1;
        Ok(())
    }

    fn declare_variable(&mut self) -> eyre::Result<()> {
        if self.scope_depth == 0 {
            return Ok(());
        }

        let name = &self.parser.previous;
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth as isize {
                break;
            }
            if name.lexeme == local.name.lexeme {
                self.error(b"Already a variable with this name in this scope.")?;
                return Ok(());
            }
        }
        let name = name.clone();
        self.add_local(name)?;
        Ok(())
    }

    fn parse_variable(&mut self, error_message: &[u8]) -> eyre::Result<u8> {
        self.consume(TokenKind::Identifier, error_message)?;

        self.declare_variable()?;
        if self.scope_depth > 0 {
            return Ok(0);
        }

        self.identifier_constant(&self.parser.previous.clone())
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope_depth > 0 {
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal, global);
    }
}

pub struct Compiler<'source, 'chunk> {
    locals: Vec<Local>,
    local_count: usize,
    scope_depth: usize,
    parser: Parser<'source, 'chunk>,
}

impl<'source, 'chunk> Compiler<'source, 'chunk> {
    fn new(parser: Parser<'source, 'chunk>) -> Self {
        Self {
            locals: Vec::with_capacity(std::u8::MAX as usize + 1),
            local_count: 0,
            scope_depth: 0,
            parser,
        }
    }
}

#[derive(Default)]
struct Local {
    name: Token,
    depth: isize,
}

pub fn compile(source: &[u8], chunk: &mut crate::chunk::Chunk) -> eyre::Result<bool> {
    let scanner = Scanner::new(source);
    let parser = Parser::new(scanner, chunk);
    let mut compiler = Compiler::new(parser);
    compiler.advance()?;

    while !compiler.matches(TokenKind::Eof)? {
        compiler.declaration()?;
    }

    compiler.end_compiler();
    Ok(!compiler.parser.had_error)
}
