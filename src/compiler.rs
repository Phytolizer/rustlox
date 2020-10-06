use std::io::Write;

use crate::{chunk::Chunk, chunk::OpCode, scanner::{Scanner, Token, TokenKind}};

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
    fn expression(&mut self) {
        todo!()
    }
    fn consume(&mut self, expected: TokenKind, message: &[u8]) -> eyre::Result<()> {
        if self.current.kind == expected {
            self.advance()?;
            return Ok(());
        }

        self.error_at_current(message)?;
        Ok(())
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

    fn end_compiler(&mut self) {
        self.emit_return();
    }
}

pub fn compile(source: &[u8], chunk: &mut crate::chunk::Chunk) -> eyre::Result<bool> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner, chunk);
    parser.advance()?;
    parser.expression();
    parser.consume(TokenKind::Eof, b"Expect end of expression.")?;
    parser.end_compiler();
    Ok(!parser.had_error)
}
