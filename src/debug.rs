use std::convert::TryFrom;

use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    match OpCode::try_from(chunk.code[offset]) {
        Ok(c) => match c {
            OpCode::Constant => constant_instruction("Constant", chunk, offset),
            OpCode::Nil => simple_instruction("Nil", offset),
            OpCode::True => simple_instruction("True", offset),
            OpCode::False => simple_instruction("False", offset),
            OpCode::Pop => simple_instruction("Pop", offset),
            OpCode::GetLocal => byte_instruction("GetLocal", chunk, offset),
            OpCode::SetLocal => byte_instruction("SetLocal", chunk, offset),
            OpCode::GetGlobal => constant_instruction("GetGlobal", chunk, offset),
            OpCode::DefineGlobal => constant_instruction("DefineGlobal", chunk, offset),
            OpCode::SetGlobal => constant_instruction("SetGlobal", chunk, offset),
            OpCode::Equal => simple_instruction("Equal", offset),
            OpCode::Greater => simple_instruction("Greater", offset),
            OpCode::Less => simple_instruction("Less", offset),
            OpCode::Add => simple_instruction("Add", offset),
            OpCode::Sub => simple_instruction("Sub", offset),
            OpCode::Mul => simple_instruction("Mul", offset),
            OpCode::Div => simple_instruction("Div", offset),
            OpCode::Not => simple_instruction("Not", offset),
            OpCode::Negate => simple_instruction("Negate", offset),
            OpCode::Print => simple_instruction("Print", offset),
            OpCode::Return => simple_instruction("Return", offset),
        },
        Err(_) => {
            println!("Unknown opcode {}", chunk.code[offset]);
            offset + 1
        }
    }
}

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    println!("{:<16} {:4}", name, slot);
    offset + 2
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    println!(
        "{:<16} {:4} '{}'",
        name, constant, chunk.constants[constant as usize]
    );
    offset + 2
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}
