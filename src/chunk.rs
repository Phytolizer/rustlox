use crate::value::Value;

#[repr(u8)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    Equal,
    Greater,
    Less,
    Add,
    Sub,
    Mul,
    Div,
    Not,
    Negate,
    Print,
    Return,
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
