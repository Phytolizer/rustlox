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

macro_rules! common_op {
    ($vm:ident, $op:tt) => {{
        if !$vm.stack.last().unwrap().is_number() || !$vm.stack[$vm.stack.len() - 2].is_number() {
            $vm.runtime_error("Operands must be numbers.");
            return InterpretResult::RuntimeError;
        }
        let b = $vm.stack.pop().unwrap();
        let a = $vm.stack.pop().unwrap();

        (a, b)
    }}
}

macro_rules! binary_op {
    ($vm:ident, $op:tt) => {{
        let (a, b) = common_op!($vm, $op);
        $vm.stack.push(a $op b);
    }};
}

macro_rules! bool_op {
    ($vm:ident, $op:tt) => {{
        let (a, b) = common_op!($vm, $op);
        $vm.stack.push(Value::Bool(a $op b));
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
        self.chunk.as_ref().unwrap().constants[offset].clone()
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
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Equal => {
                        let b = self.stack.pop().unwrap();
                        let a = self.stack.pop().unwrap();
                        self.stack.push(Value::Bool(a == b));
                    }
                    OpCode::Greater => bool_op!(self, >),
                    OpCode::Less => bool_op!(self, <),
                    OpCode::Add => binary_op!(self, +),
                    OpCode::Sub => binary_op!(self, -),
                    OpCode::Mul => binary_op!(self, *),
                    OpCode::Div => binary_op!(self, /),
                    OpCode::Not => {
                        let val = self.stack.pop().unwrap();
                        self.stack.push(Value::Bool(val.is_falsey()));
                    }
                    OpCode::Negate => {
                        let val = self.stack.last().unwrap();
                        if val.is_number() {
                            let val = self.stack.pop().unwrap();
                            self.stack.push(-val);
                        } else {
                            self.runtime_error("Operand must be a number.");
                            return InterpretResult::RuntimeError;
                        }
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

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        let instruction = self.ip - 1;
        let line = self.chunk.as_ref().unwrap().lines[instruction];
        eprintln!("[line {}] in script", line);
    }
}
