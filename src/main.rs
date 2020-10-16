mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod object;
mod parser;
mod runtime_error;
mod scanner;
mod stmt;
mod token;

use lazy_static::lazy_static;
use parser::Parser;
use scanner::Scanner;
use token::{Token, TokenKind};

use std::{
    io::{BufRead, Write},
    sync::RwLock,
};

lazy_static! {
    static ref HAD_ERROR: RwLock<bool> = RwLock::new(false);
    static ref HAD_RUNTIME_ERROR: RwLock<bool> = RwLock::new(false);
    static ref INTERPRETER: RwLock<interpreter::Interpreter> =
        RwLock::new(interpreter::Interpreter::new());
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        1 => run_prompt().unwrap(),
        2 => run_file(&args[1]).unwrap(),
        _ => {
            println!("Usage: jlox [script]");
            std::process::exit(64);
        }
    }
}

fn run_file(name: &str) -> Result<(), std::io::Error> {
    let source = std::fs::read_to_string(name)?;
    run(&source);

    if *HAD_ERROR.read().unwrap() {
        std::process::exit(65);
    }
    if *HAD_RUNTIME_ERROR.read().unwrap() {
        std::process::exit(70);
    }
    Ok(())
}

fn run_prompt() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin);
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if let Ok(0) = reader.read_line(&mut line) {
            break;
        }
        run(&line);
        *HAD_ERROR.write().unwrap() = false;
    }
    Ok(())
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    if *HAD_ERROR.read().unwrap() {
        return;
    }

    INTERPRETER
        .write()
        .unwrap()
        .interpret(statements.as_ref().unwrap());
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn error_at_token(token: &Token, message: &str) {
    if token.kind == TokenKind::Eof {
        report(token.line, " at end", message);
    } else {
        report(
            token.line,
            &(String::from(" at '") + &token.lexeme + "'"),
            message,
        );
    }
}

pub fn runtime_error(error: runtime_error::RuntimeError) {
    eprintln!("{}", error);
    *HAD_RUNTIME_ERROR.write().unwrap() = true;
}

fn report(line: usize, whence: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, whence, message);
    *HAD_ERROR.write().unwrap() = true;
}
