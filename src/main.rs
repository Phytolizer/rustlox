pub mod chunk;
pub mod debug;
pub mod value;

use chunk::{Chunk, OpCode};
use value::Value;

fn main() {
    let mut c = Chunk::new();
    c.write(OpCode::Return, 123);

    let constant = c.add_constant(Value(1.2));
    c.write(OpCode::Constant, 123);
    c.write(constant as u8, 123);

    debug::disassemble_chunk(&c, "test chunk");
}
