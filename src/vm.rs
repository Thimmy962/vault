use std::{collections::HashMap, rc::Rc};

use crate::bytes::{Bytes, OPCODE, Value};



 macro_rules! compute {
    ($opcode:expr, $a:expr, $b:expr, $type:ident) => {
        match $opcode {
            OPCODE::ADD => Value::$type($a + $b),
            OPCODE::SUB => Value::$type($a - $b),
            OPCODE::MUL => Value::$type($a * $b),
            OPCODE::MOD => {
                // Rust's partial_cmp or a simple check works for both 0 and 0.0
                if $b as f32 == 0.00 { 
                    eprintln!("Runtime Error: Division by zero."); 
                    return true; 
                }
                Value::DOUBLE($a as f32 % $b as f32)
            }
            OPCODE::DIV => {
                // Rust's partial_cmp or a simple check works for both 0 and 0.0
                if $b as f32 == 0.00 { 
                    eprintln!("Runtime Error: Division by zero."); 
                    return true; 
                }
                Value::DOUBLE($a as f32 / $b as f32)
            }
            _ => return true,
        }
    };
}

macro_rules! concatenate {
    ($opcode: expr, $a: expr, $b: expr) => {
        match $opcode {
            OPCODE::ADD => {
                let combined = Rc::new(format!("{}{}", $a, $b));
                Value::STRING(combined)
            },
            _ => {
                eprintln!("Runtime Error: String can only be concatenated.");
                return true; 
            }
        }
    };
}

macro_rules! compare {
    ($opcode:expr, $a:expr, $b:expr) => {
        match $opcode {
            OPCODE::GREATER_THAN  => $a > $b,
            OPCODE::GREATER_EQUAL => $a >= $b,
            OPCODE::LESS_THAN     => $a < $b,
            OPCODE::LESS_EQUAL    => $a <= $b,
            OPCODE::EQUAL_EQUAL   => $a == $b,
            OPCODE::BANG_EQUAL    => $a != $b,
            _ => return true,
        }
    };
}

pub struct VM{
    values: Vec<Value>,
    globalvar: HashMap<Rc<String>, Value>
}

impl VM{
    pub fn build() -> Self {
        VM {
            values: Vec::new(),
            globalvar: HashMap::new()
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
            Value::INT(val) => self.push(Value::INT(-val)),
            Value::DOUBLE(val) =>  self.push(Value::DOUBLE(-val)),
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
                return true
            }
        };

        let result = match (first, second) {
            (Value::INT(a), Value::INT(b))       => compute!(*opcode, a, b, INT),
            (Value::DOUBLE(a), Value::DOUBLE(b)) => compute!(*opcode, a, b, DOUBLE),
            (Value::DOUBLE(a), Value::INT(b))    => compute!(*opcode, a, b as f32, DOUBLE),
            (Value::INT(a), Value::DOUBLE(b))    => compute!(*opcode, a as f32, b, DOUBLE),
            (Value::STRING(a), Value::STRING(b))    => concatenate!(*opcode, a, b),
            _ => {
                    eprintln!("Runtime Error: Operands of '{:?}' must be numeric types or situation where each operands is a string .", *opcode);
                    return true;
            }
        };
        // 4. Push the calculated result value wrapper back onto the stack!
        self.push(result);
    
        false // Return false indicating no execution error occurred
    }
    

    // for the comparison opcodes >,>=, <, <=, ==
    fn comparison(&mut self, code: &OPCODE) -> bool {
        let second = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing left operand).");
                return true;
            }
        };
        let first = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing left operand).");
                return true;
            }
        };
        
    let res = match (first, second) {
        // 1. Same-type numeric comparisons
        (Value::INT(a), Value::INT(b))       => compare!(*code, a, b),
        (Value::DOUBLE(a), Value::DOUBLE(b)) => compare!(*code, a, b),

        // 2. Mixed numeric comparisons (Promote INT to f32)
        (Value::DOUBLE(a), Value::INT(b))    => compare!(*code, a, b as f32),
        (Value::INT(a), Value::DOUBLE(b))    => compare!(*code, a as f32, b),

        // 3. Text/Character comparisons (Alphabetical/ASCII order)
        (Value::CHAR(a), Value::CHAR(b))     => compare!(*code, a, b),

        // 4. Boolean comparisons (false evaluates as less than true)
        (Value::BOOL(a), Value::BOOL(b))     => compare!(*code, a, b),
        
        (Value::STRING(a), Value::STRING(b))    => compare!(*code, a, b),
        // 5. Fallback for completely invalid combinations
        // (e.g., comparing a CHAR to an INT, or anything involving NIL)
        _ => {
            eprintln!("Runtime Error: Invalid operands for ordering comparison.");
            return true;
        }
    };
        self.push(Value::BOOL(res));
        false
    }

    fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    fn push(&mut self, val: Value) {
        self.values.push(val);
    }

    fn isfalsey(&mut self) -> bool {
        let value = match self.values.pop() {
            Some(value) => value,
            None => {
                eprintln!("Runtime Error: Stack underflow (missing left operand).");
                return true;
            }    
        };

        match value {
            Value::NIL => self.push(Value::BOOL(false)),
            Value::BOOL(a) => {
                self.push(Value::BOOL(!a))
            }
            Value::STRING(string) => {
                let val = string.len() > 0;
                self.push(Value::BOOL(val))
            }
            _ => self.push(Value::BOOL(false))
        }

        false
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
                },
                OPCODE::BANG => if self.isfalsey() { return true; }

                OPCODE::CONSTANT(index) => {
                    match chunk.values().get(*index) {
                        Some(value) => {
                            self.push(value.clone());
                        },
                        None => {
                            eprintln!("No value stored");
                            return true
                        }
                    }
                },
                OPCODE::SUB | OPCODE::DIV | OPCODE::ADD | OPCODE::MUL | OPCODE::MOD => {
                    let bres = self.binary(i); // binary result
                    if bres {
                        return bres;
                    }
                },
                OPCODE::GREATER_EQUAL | OPCODE::LESS_EQUAL |
                OPCODE::LESS_THAN | OPCODE::GREATER_THAN 
                | OPCODE::EQUAL_EQUAL | OPCODE::BANG_EQUAL => {
                    if self.comparison(i) {
                        return true;
                    }
                },
                OPCODE::POP => {self.pop();},
                OPCODE::GET_GLOBAL => {
                    let key = self.pop().unwrap();
                    if let Value::STRING(ref s) = key {
                        if let Some(val) = self.globalvar.get(s) {
                            self.push(val.clone());
                            continue;
                        }
                    }
                    println!("Invalid variable name: {:?}", key);
                    return true;
                }
                OPCODE::DEF_GLOBAL  | OPCODE::SET_GLOBAL => {
                    let value = self.pop().unwrap();
                    let key = self.pop().unwrap();
                    if let Value::STRING(ref key) = key {
                        self.globalvar.insert(key.clone(), value);
                    }
                    println!("Invalid variable name: {:?}", key);
                    return true;
                },
                OPCODE::PRINT => {
                    if let Some(val) = self.values.pop() {
                        match val {
                            Value::INT(num)         => println!("{num}"),
                            Value::DOUBLE(num)      => println!("{}",num),
                            Value::CHAR(character)  => println!("{character}"),
                            Value::BOOL(val)        => println!("{val}"),
                            Value::STRING(val)      => println!("{val}"),
                            Value::NIL              => println!("NIL") 
                        }
                    } else {
                        eprintln!("Runtime Error: Stack underflow.");
                        return true;
                    }
                }
                OPCODE::EOT => return false,
                _   => todo!()
            }
        }
        false
    }
}

