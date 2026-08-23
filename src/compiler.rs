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


     fn previous(&self) -> &Token {
        return self.previous.as_ref().unwrap()
    }

    fn current(&self) -> &Token {
        return self.current.as_ref().unwrap()
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
            if self.current().kind != TokenKind::ERROR {
                break 'advance;
            }
        }
    }

    fn unary(&mut self) {
        let opcode: OPCODE;
        // match token kind to determine the opcode
        match self.previous().kind {
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
        match self.previous().kind {
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
        if self.previous().kind == TokenKind::INT {
            let val = self.previous().lexeme.clone().parse::<i32>().unwrap();
            self.write_constant(Value::INT(val));
        } else {
            let val = self.previous().lexeme.clone().parse::<f32>().unwrap();
            self.write_constant(Value::DOUBLE(val));
        }
    }


    fn consume(&mut self, token_kind: TokenKind, msg: &str) {
        if self.current().kind == token_kind {
                self.advance();
                return
            } else {
                self.error = true;
                self.error(self.current().line,
                msg.to_string());
            }
    }

    fn literal(&mut self) {
        match self.previous().kind {
            TokenKind::CHAR => {
                let val = self.previous().lexeme.clone();
                let val = val.chars().next().unwrap();
                self.write_constant(Value::CHAR(val));
            },
            TokenKind::STRING => {
                let val = self.previous().lexeme.clone();
                let a = Rc::new(val);
                self.write_constant(Value::STRING(a));
            },
            TokenKind::FALSE => self.write_constant(Value::BOOL(false)),
            TokenKind::TRUE => self.write_constant(Value::BOOL(true)),
            _ => return
        }
    }

    fn variable(&mut self) {
        let val = Rc::new(self.previous().lexeme.clone());
        self.write_constant(Value::STRING(val));
        
        if self.match_token(TokenKind::EQUAL) {
            self.expression();
            self.write(OPCODE::SET_GLOBAL);
        } else {
            self.write(OPCODE::GET_GLOBAL)
        }
        
    }

    //handles bracket
    fn bracket(&mut self) {
        self.parseprecedence(PRECEDENCE::Assignment);
        self.consume(TokenKind::CBRACKET, "Expects a ')'");
    }

    // get a prefix for a token; if such token has no prefix it returns None
    fn getprefix(&self) -> Option<fn(&mut Self)> {
        match self.previous().kind {
            TokenKind::MINUS | TokenKind::BANG => Some(Self::unary),
            TokenKind::INT | TokenKind::DOUBLE => Some(Self::number),
            TokenKind::STRING | TokenKind::CHAR |TokenKind::FALSE | TokenKind::TRUE => Some(Self::literal),
            TokenKind::OBRACKET => Some(Self::bracket),
            TokenKind::IDENTIFIER => Some(Self::variable),
            _ => None,
        }
    }

    // returns the precedence level of current token
    fn getcurrentprecedence(&self) -> PRECEDENCE {
         match self.current().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR => PRECEDENCE::Factor,
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL => PRECEDENCE::Comparison,
            _ => PRECEDENCE::None,
        } 
    }


 // returns the precedence level of previous token
    fn getpreviousprecedence(&self) -> PRECEDENCE {
         match self.previous().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR => PRECEDENCE::Factor,
            _ => PRECEDENCE::None,
        }
 
    }

    fn getinfix(&self) -> Option<fn(&mut Self)> {
         match self.previous().kind {
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
    
    fn error(&mut self, line: usize, message: String) {
        self.error = true;
        eprintln!("[line {}]{}", line, message);
    }


    fn parseVariable(&mut self, msg: &str) {
        self.consume(TokenKind::IDENTIFIER, msg);
        let val = self.previous().lexeme.clone();
        let a = Rc::new(val);
        self.write_constant(Value::STRING(a));
    }

    fn varDeclaration(&mut self) {
        self.parseVariable("Expects a variable name");
        if self.match_token(TokenKind::EQUAL) {
            self.expression();
        } else {
            self.write_constant(Value::NIL);
        }

        self.consume(TokenKind::SEMI_COLON, "Syntax error: ';' is expected after declaration");
        self.write(OPCODE::DEF_GLOBAL);
    }

    //checks the current token against an expected token
    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if !(self.current().kind == kind) {
            return false;
        }

        self.advance();
        true
    }

    fn synchronize(&mut self) {
        self.error = false;
        while !self.match_token(TokenKind::EOT) {
            match self.current().kind {
                TokenKind::SEMI_COLON | TokenKind::LET | TokenKind::STRING | TokenKind::PRINT
                | TokenKind::INT | TokenKind::DOUBLE | TokenKind::CHAR
                    => break,
                _ => self.advance(),
            }
        }
    }

    pub fn declaration(&mut self) {
        match self.current().kind {
            TokenKind::LET => {
                self.advance();
                self.varDeclaration();
            }
            _ => self.statement()
        }

        if self.error {self.synchronize();}
    }

    fn statement(&mut self) {
        match self.current().kind {
            TokenKind::PRINT => {
                self.advance();
                self.printstatement();
            },
            _ => {
                self.expressionStatement();
            }
        }
    }


    // for an expression
    fn expressionStatement(&mut self) {
        self.expression();
        self.consume(TokenKind::SEMI_COLON, "Expect ';' after expression.");
        self.write(OPCODE::POP);
    }


    fn expect_token(&mut self, token: TokenKind) {
            }

    // print statement
    fn printstatement(&mut self) {
        self.expression();
        self.consume(TokenKind::SEMI_COLON, "Expects ';' after value");
        self.write(OPCODE::PRINT);
    }

    
    pub fn parseprecedence(&mut self, precedence: PRECEDENCE) {
        self.advance();
        match self.getprefix() {
            Some(func) => func(self),
            None => {
                self.error(self.previous().line, 
                    "Syntax error: '".to_owned() + &self.previous().lexeme + "' is not a valid syntax");
            }
        }

        while self.current().kind != TokenKind::EOT && self.getcurrentprecedence() >= precedence {
            self.advance();
            match self.getinfix() {
                Some(func) => func(self),
                None =>  {
                    self.error(self.previous().line,
                        "Syntax error: '".to_owned() + &self.previous().lexeme + "' is not a valid syntax");
                    break;
                }
            }
        }
    }
}


pub fn compile<'a>(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes) -> bool {
    let mut parser = Parser::build(scanner, bytecodes);
    parser.advance();

    while !parser.match_token(TokenKind::EOT) {
        parser.declaration();
    }

    parser.endcompiler();
    parser.error
}
