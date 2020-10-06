#[repr(u8)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum OpCode {
    Return,
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B) {
        self.code.push(byte.into());
    }
}
