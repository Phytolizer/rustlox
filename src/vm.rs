use std::convert::TryFrom;

use crate::{
    chunk::{Chunk, OpCode},
    common::DEBUG_TRACE_EXECUTION,
    debug::disassemble_instruction,
    value::Value,
};

#[derive(Default)]
pub struct VM<'c> {
    pub chunk: Option<&'c Chunk>,
    ip: usize,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl<'c> VM<'c> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, chunk: &'c Chunk) -> InterpretResult {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.unwrap().code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.unwrap().constants[self.read_byte() as usize]
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                disassemble_instruction(self.chunk.as_ref().unwrap(), self.ip);
            }
            match OpCode::try_from(self.read_byte()) {
                Ok(oc) => match oc {
                    OpCode::Constant => {
                        let constant = self.read_constant();
                        constant.print();
                        println!();
                    }
                    OpCode::Return => {
                        return InterpretResult::Ok;
                    }
                },
                Err(_) => {}
            }
        }
    }
}
