use crate::bytes::{Bytes, OPCODE, Value};


// should reference the contents of Bytes
pub struct Debug <'a> {
    codes: &'a Vec<OPCODE>,
    values: &'a Vec<Value>
}

impl <'a> Debug <'a> {
    pub fn new(chunk: &'a Bytes) -> Debug<'a> {
        Debug {codes: chunk.codes(), values: chunk.values() }
    }

    // loops the the opcodes and prints their values as a form of debugging
    pub fn run(&self) {
        println!("------------DEBUGGING---------");
        for code in self.codes {
            match code {
                OPCODE::CONSTANT(index) => {
                    self.constant(*index);
                },
                OPCODE::NEGATE      => self.instruction("-"),
                OPCODE::SUB         => self.instruction("-"),
                OPCODE::DIV         => self.instruction("/"),
                OPCODE::MUL         => self.instruction("*"),
                OPCODE::ADD         => self.instruction("+")
            }
        }
    }

    pub fn instruction(&self, val: &str)  {
        println!("Instruction      ||          {val}")
    }

    // print constants
    fn constant(&self, index: usize) {
        match self.values.get(index) { 
            Some(value)=> value.print_value(),
            None => println!("No value is stored")
        }
    }
}
