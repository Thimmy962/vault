use crate::bytes::{Bytes, OPCODE, Value};

pub struct VM{
    values: Vec<Value>,
}

impl VM{
    pub fn build() -> Self {
        VM {
            values: Vec::new()
        }
    }
    fn negate(&mut self) -> bool {
        let value = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing left operand).");
                return true;
            }    
        };
        
        match value {
            Value::INT(val) => self.values.push(Value::INT(-val)),
            Value::DOUBLE(val) =>  self.values.push(Value::DOUBLE(-val)),
            _ => {
                eprintln!("Runtime Error: Operands must be numeric types.");
                return true;
            }
        }

        false
    }

    fn binary(&mut self, opcode: &OPCODE) -> bool {
        // 1. Pop the right hand side (second operand)
        let second = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing right operand).");
                return true;
            }
        };
    
        // 2. Pop the left hand side (first operand)
        let first = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing left operand).");
                return true;
            }
        };

        // 3. Match against BOTH variants at the same time to determine calculations dynamically
        let result = match (first, second) {
                       // Handle both integers
            (Value::INT(a), Value::INT(b)) => {
                match opcode {
                    OPCODE::ADD => Value::INT(a + b),
                    OPCODE::SUB => Value::INT(a - b),
                    OPCODE::MUL => Value::INT(a * b),
                    OPCODE::DIV => {
                        if b == 0 {
                            eprintln!("Runtime Error: Division by zero.");
                            return true;
                        }
                        Value::INT(a / b)
                    }
                    _ => return true,
                }
            }
            // Handle both doubles
            (Value::DOUBLE(a), Value::DOUBLE(b)) => {
                match opcode{
                    OPCODE::ADD => Value::DOUBLE(a + b),
                    OPCODE::SUB => Value::DOUBLE(a - b),
                    OPCODE::MUL => Value::DOUBLE(a * b),
                    OPCODE::DIV => Value::DOUBLE(a / b),
                    _ => return true,
                }
            }
            // Mixed math: Float and Integer (Cast integer to float)
            (Value::DOUBLE(a), Value::INT(b)) => {
                let b_float = b as f32;
                match opcode {
                    OPCODE::ADD => Value::DOUBLE(a + b_float),
                    OPCODE::SUB => Value::DOUBLE(a - b_float),
                    OPCODE::MUL => Value::DOUBLE(a * b_float),
                    OPCODE::DIV => Value::DOUBLE(a / b_float),
                    _ => return true,
                }
            }
            // Mixed math: Integer and Float (Cast integer to float)
            (Value::INT(a), Value::DOUBLE(b)) => {
                let a_float = a as f32;
                match opcode {
                    OPCODE::ADD => Value::DOUBLE(a_float + b),
                    OPCODE::SUB => Value::DOUBLE(a_float - b),
                    OPCODE::MUL => Value::DOUBLE(a_float * b),
                    OPCODE::DIV => Value::DOUBLE(a_float / b),
                    _ => return true,
                }
            }
            // Fallback for non-numeric types (like NIL, BOOL, etc.)
            _ => {
                eprintln!("Runtime Error: Operands must be numeric types.");
                return true;
            }
        };

    // 4. Push the calculated result value wrapper back onto the stack!
        self.values.push(result);
    
        false // Return false indicating no execution error occurred
    }


    // returns true for a runtime error
    // false for no error
    pub fn run(&mut self, chunk: &Bytes) ->  bool {
        for i in chunk.codes() {
            match i {
                OPCODE::NEGATE => {
                    if self.negate() {
                        return true;
                    }
                }
                OPCODE::CONSTANT(index) => {
                    match chunk.values().get(*index) {
                        Some(value) => {
                            self.values.push(value.clone());
                        },
                        None => {
                            eprintln!("No value stored");
                            return true
                        }
                    }
                },
                OPCODE::SUB | OPCODE::DIV | OPCODE::ADD | OPCODE::MUL => {
                    let bres = self.binary(i); // binary result
                    if bres {
                        return bres;
                    }
                },
                OPCODE::EOT => {
                    if let Some(val) = self.values.pop() {
                        match val {
                            Value::INT(num) => println!("{num}"),
                            Value::DOUBLE(num) => println!("{num}"),
                            _ => {
                                eprintln!("Runtime Error: Operands must be numeric types.");
                                return true;
                            }
                        }
                    } else {
                        eprintln!("Runtime Error: Stack underflow.");
                        return true;
                    }
                }
            }
        }
        false
    }
}

