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
                OPCODE::NEGATE          => self.instruction("Negate", "-"),
                OPCODE::SUB             => self.instruction("Subtract", "-"),
                OPCODE::MOD             => self.instruction("Modulus", "//"),
                OPCODE::DIV             => self.instruction("Divide", "/"),
                OPCODE::MUL             => self.instruction("Multiply", "*"),
                OPCODE::ADD             => self.instruction("Addition", "+"),
                OPCODE::EOT             => return,
                OPCODE::GREATER_THAN    => self.instruction("Greater Than", ">"),
                OPCODE::GREATER_EQUAL   => self.instruction("Greater or Equal", ">="),
                OPCODE::LESS_EQUAL      => self.instruction("Less or Equal", "<="),
                OPCODE::LESS_THAN       => self.instruction("Less Than", "<"),
                OPCODE::EQUAL           => self.instruction("Assignemnt", "="),
                OPCODE::EQUAL_EQUAL     => self.instruction("Equality", "=="),
                OPCODE::BANG            => self.instruction("Bang", "!"),
                OPCODE::BANG_EQUAL      => self.instruction("Not Equal", "!="),
                OPCODE::PRINT           => self.instruction("Print", "  "),
                OPCODE::POP             => self.instruction("pop", "pop"),
                OPCODE::DEF_GLOBAL      => self.instruction("Global variable","GLOBAL"),
                OPCODE::SET_GLOBAL      => self.instruction("Set Global", "GLOBAL"),
                OPCODE::GET_GLOBAL      => self.instruction("Get Global", "GLOBAL"),
                OPCODE::JUMP_IF_FALSE(point)   => self.instruction("Jump if false", &point.to_string()[..]),
                OPCODE::JUMP(point)            =>  self.instruction("Jump", &point.to_string()[..]),

                _                       => continue,
            }
        }
    }

    pub fn instruction(&self, instruction: &str, val: &str)  {
        println!("{:<12}     ||          {}", instruction, val);
    }

    // print constants
    fn constant(&self, index: usize) {
        match self.values.get(index) { 
            Some(value)=> value.print_value(),
            None => println!("No value is stored")
        }
    }
}
