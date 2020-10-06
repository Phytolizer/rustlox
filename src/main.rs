pub mod chunk;
pub mod common;
pub mod debug;
pub mod value;
pub mod vm;

use chunk::{Chunk, OpCode};
use value::Value;
use vm::VM;

fn main() {
    let mut vm = VM::new();

    let mut c = Chunk::new();
    let constant = c.add_constant(Value(1.2));
    c.write(OpCode::Constant, 123);
    c.write(constant as u8, 123);

    c.write(OpCode::Negate, 123);

    c.write(OpCode::Return, 123);

    debug::disassemble_chunk(&c, "test chunk");
    vm.interpret(&c);
}
