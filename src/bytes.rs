use std::rc::Rc;

pub trait IsNumber {}
impl IsNumber for  i32{}
impl IsNumber for  f32{}



#[derive(Debug, PartialEq, Clone)]
pub enum Value{
    INT(i32), DOUBLE(f32), BOOL(bool), CHAR(char), STRING(Rc<String>), NIL
}

impl  Value {
    pub fn print_value(&self) {
        let label = "CONSTANT";
        match self {
            Value::INT(val)    => println!("{:<16} || int      {}", label, val),
            Value::DOUBLE(val) => println!("{:<16} || float    {}", label, val),
            Value::BOOL(val)   => println!("{:<16} || bool     {}", label, val),
            Value::CHAR(val)   => println!("{:<16} || char     {}", label, val),
            Value::STRING(val)   => println!("{:<16} || char     {}", label, val),
            Value::NIL         => println!("{:<16} || nil      NIL", label),
        }
    } 
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum OPCODE {
    SUB, ADD, MUL, DIV, MOD, NEGATE, CONSTANT(usize),// houses the 
    GREATER_EQUAL, LESS_EQUAL, 
    LESS_THAN, GREATER_THAN,
    EQUAL, EQUAL_EQUAL, POP, 
    DEF_GLOBAL, SET_GLOBAL, GET_GLOBAL,
    BANG, BANG_EQUAL, PRINT, EOT
}


pub struct Bytes {
    codes: Vec<OPCODE>,
    values: Vec<Value>
}

impl Bytes {
    // build a new instance of the Bytes struct
    pub fn  new() -> Bytes {
        Bytes { codes: Vec::new(), values: Vec::new() }
    }
    
    // returns a reference to the vector holding the opcodes
    pub fn codes(&self) -> &Vec<OPCODE> {
        &self.codes
    }

    // returns a reference to the vector holding the constants
    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }
    


    pub fn write(&mut self, code: OPCODE) {
        self.codes.push(code);
    }

    pub fn write_constant(&mut self, value: Value) {
        // push the value into the Value vec
        self.values.push(value);
        // get the position of the last pushed Value
        let pos: usize = self.values.len() - 1;
        //create an constant with the associated index as the position of the constant written to
        //the Value vector of bytes
        self.codes.push(OPCODE::CONSTANT(pos));
    }



    pub fn clear(&mut self) {
        self.values.clear();
        self.codes.clear();
    }
}
