use std::rc::Rc;

use crate::bytes::{Bytes, OPCODE, Value};
use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};


#[derive(PartialEq, Clone, Copy, Debug, PartialOrd, Ord, Eq)]
pub enum PRECEDENCE {
    None, 
    Assignment, // =
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
            PRECEDENCE::None            => PRECEDENCE::Assignment,
            PRECEDENCE::Assignment     => PRECEDENCE::Or,
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

    fn write_constant(&mut self, value: Value) {
        self.bytecodes.write_constant(value);
    }


    // write an opcode that has no constant value to vec of bytecode
    fn write(&mut self, code: OPCODE) {
        self.bytecodes.write(code);
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

    fn unary(&mut self) {
        let opcode: OPCODE;
        // match token kind to determine the opcode
        match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS => opcode = OPCODE::NEGATE,
            TokenKind::BANG => opcode = OPCODE::BANG,
            _ => return
        }

        self.parseprecedence(PRECEDENCE::Unary);

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
            TokenKind::GREATER_THAN => opcode = OPCODE::GREATER_THAN,
            TokenKind::GREATER_EQUAL => opcode = OPCODE::GREATER_EQUAL,
            TokenKind::LESS_THAN => opcode = OPCODE::LESS_THAN,
            TokenKind::LESS_EQUAL => opcode = OPCODE::LESS_EQUAL,
            TokenKind::EQUAL_EQUAL => opcode = OPCODE::EQUAL_EQUAL,
            TokenKind::BANG_EQUAL => opcode = OPCODE::BANG_EQUAL,
            _ => return
        }

        let prec = self.getpreviousprecedence();

        self.parseprecedence(prec.next());   

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


    fn consume(&mut self, token_kind: TokenKind, msg: &str) {
        if self.current.as_ref().unwrap().kind == token_kind {
                self.advance();
                return
            } else {
                self.error = true;
                self.error(msg.to_string());
            }
    }

    fn literal(&mut self) {
        match self.previous.as_ref().unwrap().kind {
            TokenKind::CHAR => {
                let val = self.previous.as_ref().unwrap().lexeme.clone();
                let val = val.chars().next().unwrap();
                self.write_constant(Value::CHAR(val));
            },
            TokenKind::STRING => {
                let val = self.previous.as_ref().unwrap().lexeme.clone();
                let a = Rc::new(val);
                self.write_constant(Value::STRING(a));
            },
            TokenKind::FALSE => self.write_constant(Value::BOOL(false)),
            TokenKind::TRUE => self.write_constant(Value::BOOL(true)),
            _ => return
        }
    }

    fn bracket(&mut self) {
        self.parseprecedence(PRECEDENCE::Assignment);
        self.consume(TokenKind::CBRACKET, "Expects a ')'");
    }

    // get a prefix for a token; if such token has no prefix it returns None
    fn getprefix(&self) -> Option<fn(&mut Self)> {
        match self.previous.as_ref().unwrap().kind {
            TokenKind::MINUS | TokenKind::BANG => Some(Self::unary),
            TokenKind::INT | TokenKind::DOUBLE => Some(Self::number),
            TokenKind::STRING | TokenKind::CHAR |TokenKind::FALSE | TokenKind::TRUE => Some(Self::literal),
            TokenKind::OBRACKET => Some(Self::bracket),
            _ => None,
        }
    }

    // returns the precedence level of current token
    fn getcurrentprecedence(&self) -> PRECEDENCE {
         match self.current.as_ref().unwrap().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR => PRECEDENCE::Factor,
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL => PRECEDENCE::Comparison,
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
            TokenKind::MINUS | TokenKind::PLUS | TokenKind::STAR | TokenKind::SLASH |
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL
               => Some(Self::binary),

            _ => None,
        }
    }

    pub fn expression(&mut self) {
        self.parseprecedence(PRECEDENCE::Assignment);
    }


    pub fn endcompiler(&mut self) {
        self.consume(TokenKind::EOT, "Expects end of token");
        self.write(OPCODE::EOT);
    }
    
    fn error(&mut self, message: String) {
        self.error = true;
        eprintln!("{message}")
    }

    pub fn parseprecedence(&mut self, precedence: PRECEDENCE) {
        self.advance();
        match self.getprefix() {
            Some(func) => func(self),
            None => {
                self.error("Syntax error: '".to_owned() + &self.previous.as_ref().unwrap().lexeme + "' is not a valid syntax");
            }
        }

        while self.current.as_ref().unwrap().kind != TokenKind::EOT && self.getcurrentprecedence() >= precedence {
            self.advance();
            match self.getinfix() {
                Some(func) => func(self),
                None =>  {
                    self.error("Syntax error: '".to_owned() + &self.previous.as_ref().unwrap().lexeme + "' is not a valid syntax");
                    break;
                }
            }
        }
    }
}


pub fn compile<'a>(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes) -> bool {
    let mut parser = Parser::build(scanner, bytecodes);
    parser.advance();
    parser.expression();
    parser.endcompiler();
    parser.error
}
