use crate::value::Value;

#[repr(u8)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum OpCode {
    Constant,
    Return,
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B) {
        self.code.push(byte.into());
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
