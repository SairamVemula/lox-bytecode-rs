use crate::value::Value;

use super::value::ValueArray;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpCode {
    Constant = 0,
    Return,
    Negate,
    Add,
    Subtract,
    Multiple,
    Divide,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::Constant,
            1 => OpCode::Return,
            2 => OpCode::Negate,
            3 => OpCode::Add,
            4 => OpCode::Subtract,
            5 => OpCode::Multiple,
            6 => OpCode::Divide,
            _ => unimplemented!("Invalid opcode"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
            lines: Vec::new(),
        }
    }
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }
    pub fn write_opcode(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn read(&self, ip: usize) -> u8 {
        self.code[ip]
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
        self.constants.free();
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value)
    }

    pub fn get_constant(&self, ip: usize) -> Value {
        self.constants.get_value(ip)
    }

    pub fn disassemble<T: ToString>(&self, name: T) {
        println!("== {} ==", name.to_string());

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset])
        }

        let instruction = self.code[offset].into();
        match instruction {
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::Add => self.simple_instruction("OP_ADD", offset),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiple => self.simple_instruction("OP_MULTIPLE", offset),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset),
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{name:16} {constant:4} '");
        self.constants.print_value(constant.into());
        println!("'");
        offset + 2
    }
}
