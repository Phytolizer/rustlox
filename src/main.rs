pub mod chunk;
pub mod debug;
pub mod value;

use chunk::{Chunk, OpCode};
use value::Value;

fn main() {
    let mut c = Chunk::new();
    c.write(OpCode::Return);

    let constant = c.add_constant(Value(1.2));
    c.write(OpCode::Constant);
    c.write(constant as u8);

    debug::disassemble_chunk(&c, "test chunk");
}
