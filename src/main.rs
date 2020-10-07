pub mod chunk;
pub mod common;
pub mod compiler;
pub mod debug;
pub mod object;
pub mod scanner;
pub mod value;
pub mod vm;

use std::{
    ffi::OsString,
    io::Write,
    io::{self, BufRead, Read},
    process::exit,
};

use vm::{InterpretResult, VM};

fn main() {
    let mut vm = VM::new();

    let args: Vec<OsString> = std::env::args_os().collect();

    match args.len() {
        1 => {
            if let Err(e) = repl(&mut vm) {
                eprintln!("Fatal error: {}", e);
            }
        }
        2 => {
            if let Err(e) = run_file(&mut vm, &args[1]) {
                eprintln!("Fatal error: {}", e);
            }
        }
        _ => {
            eprintln!("Usage: clox [path]");
            exit(64);
        }
    }
}

fn repl(vm: &mut VM) -> eyre::Result<()> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    loop {
        let mut line = vec![];
        print!("> ");
        io::stdout().flush()?;

        match stdin_reader.read_until(b'\n', &mut line)? {
            0 => {
                println!();
                break;
            }
            _ => {
                vm.interpret(&line)?;
            }
        }
    }
    Ok(())
}

fn run_file(vm: &mut VM, file_name: &OsString) -> eyre::Result<()> {
    let source = {
        let mut source = vec![];
        let mut f = std::fs::File::open(&file_name)?;
        f.read_to_end(&mut source)?;
        source
    };

    match vm.interpret(&source)? {
        InterpretResult::Ok => {}
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
    Ok(())
}
