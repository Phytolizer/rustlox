use std::io::Write;

use crate::scanner::{Scanner, TokenKind};

pub fn compile(source: &[u8]) -> eyre::Result<()> {
    let mut scanner = Scanner::new(source);
    let mut line = None;
    loop {
        let token = scanner.scan_token();
        if line.is_none() || token.line != line.unwrap() {
            print!("{:4} ", token.line);
            line = Some(token.line);
        } else {
            print!("   | ");
        }

        print!("{:2} '", token.kind as u8);
        std::io::stdout().write_all(&token.lexeme)?;
        println!("'");

        if token.kind == TokenKind::Eof {
            break;
        }
    }
    Ok(())
}
