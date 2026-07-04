use crate::bytes::{Bytes, OPCODE, Value};
use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};


#[derive(PartialEq, Clone, Copy, Debug, PartialOrd, Ord, Eq)]
pub enum PRECEDENCE {
    None, 
    Asssignment, // =
    Or, // or
    And, // and
    Equality, // ==
    Comparison,// < > <= >=
    Term,// + -
    Factor,// * /
    Unary,// ! -
    Call,// . ()
    Primary
}


impl PRECEDENCE {
    pub fn next(&self)-> Self {
        match self {
            PRECEDENCE::None            => PRECEDENCE::Asssignment,
            PRECEDENCE::Asssignment     => PRECEDENCE::Or,
            PRECEDENCE::Or              => PRECEDENCE::And,
            PRECEDENCE::And             => PRECEDENCE::Equality,
            PRECEDENCE::Equality        => PRECEDENCE::Comparison,
            PRECEDENCE::Comparison      => PRECEDENCE::Term,
            PRECEDENCE::Term            => PRECEDENCE::Factor,
            PRECEDENCE::Factor          => PRECEDENCE::Unary,
            PRECEDENCE::Unary           => PRECEDENCE::Call,
            PRECEDENCE::Call            => PRECEDENCE::Primary,
            PRECEDENCE::Primary         => PRECEDENCE::Primary

        }
    }
}


pub struct Parser <'a>{
    bytecodes: &'a mut Bytes,
    scanner: &'a mut Scanner,
    previous: Option<Token>,
    current: Option<Token>,
    error: bool,
}

impl <'a>Parser <'a>{
    pub fn build(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes)-> Self {
        Parser {
        scanner, bytecodes,
        previous: None, current: None, error: false
        }
    }
   
    fn advance(&mut self) {
        self.previous = self.current.take();
        'advance: loop {
            self.current = Some(self.scanner.scan_token());
            if self.current.as_ref().unwrap().kind != TokenKind::ERROR {
                break 'advance;
            }
        }
    }

    // write an opcode that has no constant value to vec of bytecode
    fn write(&mut self, code: OPCODE) {
        self.bytecodes.write(code);
    }

    // get a prefix for a token; if such token has no prefix it returns None
    fn getprefix(&self) -> Option<fn(&mut Self)> {
        match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS => Some(Self::unary),
            TokenKind::INT | TokenKind::DOUBLE=> Some(Self::number),
            _ => None,
        }
    }

    // returns the precedence level of current token
    fn getcurrentprecedence(&self) -> PRECEDENCE {
         match self.current.as_ref().unwrap().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR => PRECEDENCE::Factor,
            _ => PRECEDENCE::None,
        } 
    }


 // returns the precedence level of previous token
    fn getpreviousprecedence(&self) -> PRECEDENCE {
         match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR => PRECEDENCE::Factor,
            _ => PRECEDENCE::None,
        }
 
    }

    fn getinfix(&self) -> Option<fn(&mut Self)> {
         match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS | TokenKind::PLUS | TokenKind::STAR | TokenKind::SLASH
                => Some(Self::binary),
            _ => None,
        }
    }

    fn write_constant(&mut self, value: Value) {
        self.bytecodes.write_constant(value);
    }

    pub fn expression(&mut self) {
        self.parserexpression(PRECEDENCE::None);
    }

    pub fn endcompiler(&mut self) {
        unimplemented!();
    }
    
    fn error(&mut self, message: String) {
        self.error = true;
        eprintln!("{message}")
    }

    pub fn parserexpression(&mut self, precedence: PRECEDENCE) {
        self.advance();
        match self.getprefix() {
            Some(func) => func(self),
            None => {
                self.error("Syntax error: '".to_owned() + &self.previous.as_ref().unwrap().lexeme + "' is not a valid syntax");
            }
        }
        /*
        while  self.getcurrentprecedence() as u8 >= precedence.clone()  as u8 {
            self.advance();
            match self.getinfix() {
                Some(func) => func(self),
                None => {
                    eprintln!("Syntax error: '{}' is not a valid syntax", self.current.as_ref().unwrap().lexeme);
                }
            }
        }*/
    }

    fn unary(&mut self) {
        let opcode: OPCODE;
        // match token kind to determine the opcode
        match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS => opcode = OPCODE::NEGATE,
            _ => return
        }

        self.parserexpression(PRECEDENCE::Unary);

        self.bytecodes.write(opcode);
    }


    fn binary(&mut self) {
        let opcode: OPCODE;
        // match token kind to determine the opcode
        match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS => opcode = OPCODE::SUB,
            TokenKind::PLUS => opcode = OPCODE::ADD,
            TokenKind::STAR => opcode = OPCODE::MUL,
            TokenKind::SLASH => opcode = OPCODE::DIV,
            _ => return
        }
        
        let prec = self.getpreviousprecedence();

        self.parserexpression(prec.next());   

        self.bytecodes.write(opcode);
    }

    fn number(&mut self) {
        if self.previous.as_ref().unwrap().kind == TokenKind::INT {
            let val = self.previous.as_ref().unwrap().lexeme.clone().parse::<i32>().unwrap();
            self.write_constant(Value::INT(val));
        } else {
            let val = self.previous.as_ref().unwrap().lexeme.clone().parse::<f32>().unwrap();
            self.write_constant(Value::DOUBLE(val));
        }
    }
}


pub fn compile<'a>(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes) -> bool {
    let mut parser = Parser::build(scanner, bytecodes);
    parser.advance();
    parser.expression();
    parser.error
}
