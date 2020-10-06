pub mod chunk;
pub mod debug;

use chunk::{Chunk, OpCode};

fn main() {
    let mut c = Chunk::new();
    c.write(OpCode::Return);

    debug::disassemble_chunk(&c, "test chunk");
}
