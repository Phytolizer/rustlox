use std::convert::TryFrom;

use crate::{
    chunk::{Chunk, OpCode},
    common::DEBUG_TRACE_EXECUTION,
    compiler::compile,
    debug::disassemble_instruction,
    value::Value,
};

#[derive(Default)]
pub struct VM {
    pub chunk: Option<Box<Chunk>>,
    ip: usize,
    stack: Vec<Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

macro_rules! binary_op {
    ($vm:ident, $op:tt) => {{
        let b = $vm.stack.pop().unwrap();
        let a = $vm.stack.pop().unwrap();

        $vm.stack.push(a $op b);
    }};
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, source: &[u8]) -> eyre::Result<InterpretResult> {
        let mut chunk = Chunk::new();
        if !compile(source, &mut chunk)? {
            return Ok(InterpretResult::CompileError);
        }

        self.chunk = Some(Box::new(chunk));
        self.ip = 0;

        Ok(self.run())
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.as_ref().unwrap().code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let offset = self.read_byte() as usize;
        self.chunk.as_ref().unwrap().constants[offset]
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {} ]", slot);
                }
                println!();
                disassemble_instruction(self.chunk.as_ref().unwrap(), self.ip);
            }
            if let Ok(oc) = OpCode::try_from(self.read_byte()) {
                match oc {
                    OpCode::Constant => {
                        let constant = self.read_constant();
                        self.stack.push(constant);
                    }
                    OpCode::Add => binary_op!(self, +),
                    OpCode::Sub => binary_op!(self, -),
                    OpCode::Mul => binary_op!(self, *),
                    OpCode::Div => binary_op!(self, /),
                    OpCode::Negate => {
                        let val = -self.stack.pop().unwrap();
                        self.stack.push(val);
                    }
                    OpCode::Return => {
                        if let Some(top) = self.stack.pop() {
                            println!("{}", top);
                        }
                        return InterpretResult::Ok;
                    }
                }
            }
        }
    }
}
