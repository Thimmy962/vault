pub enum Value{
    INT(i32), DOUBLE(f32), BOOL(bool), NIL
}

impl  Value {
    pub fn print_value(&self) {
        match self {
                Value::INT(val) =>      println!("CONSTANT       || int     {val}"),
                Value::DOUBLE(val) =>   println!("CONSTANT       || float   {val}"),
                Value::BOOL(val) =>     println!("CONSTANT       || bool    {val}"),
                Value::NIL =>           println!("CONSTANT       || nil     NIL"),
        }
    }
}

pub enum OPCODE {
    CONSTANT(usize) // houses the 
}


pub struct Bytes {
    codes: Vec<OPCODE>,
    values: Vec<Value>
}

impl Bytes {
    // build a new instance of the Bytes struct
    pub fn new() -> Bytes {
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


    pub fn write_constant(&mut self, value: Value) {
        // push the value into the Value vec
        self.values.push(value);
        // get the position of the last pushed Value
        let pos: usize = self.values.len() - 1;
        //create an constant with the associated index as the position of the constant written to
        //the Value vector of bytes
        self.codes.push(OPCODE::CONSTANT(pos));
    }
}
