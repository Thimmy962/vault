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

pub struct Local {
    name: String, // variable name
    depth: i8 // the depth where the variable was declared
}

impl Local {
    pub fn new(name: String) -> Self {
        Local {
            depth: -1, name
        }
    }
}

pub struct Locals {
    locals: Vec<Local>, // list of references to all local variables
    depth: i8 // the current depth where the copiler is.
}

impl Locals {
    pub fn new() -> Self {
        Locals {locals: Vec::new(), depth: 0}
    }
}

/// The main parser for transforming source tokens into bytecode instructions.
/// 
/// ### Error Handling Strategy
/// To prevent a single syntax error from stopping compilation or causing a wall 
/// of misleading follow-up messages, two error flags are used:
/// 
/// - `error`: A transient flag representing **panic mode**. When set, it indicates 
///   that the parser encountered an error and is currently in an invalid state. 
///   While `error` is `true`, subsequent syntax errors are suppressed to prevent 
///   cascading error noise. It is reset to `false` during synchronization when a 
///   safe statement boundary (like `;` or `let`) is reached, allowing compilation 
///   to resume and discover further distinct errors.
/// 
/// - `hadError`: A sticky flag tracking overall compilation success. Unlike `error`, 
///   once `hadError` becomes `true`, it remains `true` for the rest of the parsing 
///   pipeline. It acts as the final source of truth to inform the compiler whether 
///   valid bytecode was produced or if execution should be aborted.
pub struct Parser <'a>{
    bytecodes: &'a mut Bytes,
    scanner: &'a mut Scanner,
    previous: Option<Token>,
    current: Option<Token>, 
    locals: Locals, // list of local Variables
    error: bool, 
    hadError: bool
}

impl <'a>Parser <'a>{
    pub fn build(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes)-> Self {
        Parser {
            scanner, bytecodes,
            previous: None, current: None, error: false, hadError: false,
            locals: Locals::new()
        }
    }

    fn addLocal(locals: &mut Locals, name: String) {
        locals.locals.push(Local::new(name));
    }


    fn resolveLocalVariables(&mut self, name: &str) -> Option<usize>{
        let line = self.previous().line;
        for (index, val) in self.locals.locals.iter().rev().enumerate() {
            if val.depth != self.locals.depth{
                break;
            } else if val.name == name{
                if val.depth == -1 {
                    self.error(line, "Can't read local variable for its own initialization".to_string());
                }
                   return Some(index);
            }
        }
        None
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
            TokenKind::MOD => opcode = OPCODE::MOD,
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

    fn declareLocal(&mut self, name: &str) -> bool {
        if self.locals.depth == 0 {
            return false;
        }
        let name_str = name.to_string();
        let mut isDuplicate = false;

        for local in self.locals.locals.iter().rev() {
            if local.depth == -1 && local.depth < self.locals.depth {
                break;
            }
            if local.name == name_str {
                isDuplicate = true;
                break;
            }
        }
        if isDuplicate {
            let line = self.previous().line;
            self.error(line, "Variable with this name already exists".to_string());
        }
        Parser::addLocal(&mut self.locals, name_str);
        true
    }

    fn block(&mut self) {
        while !self.match_token(TokenKind::CCURLY) && !self.match_token(TokenKind::EOT) {
            self.declaration();
        }

        self.consume(TokenKind::CCURLY, "Expects '}' to end this scope");
    }


    fn endScope(&mut self) {
        self.locals.depth -= 1;
        while let Some(local) = self.locals.locals.last() {
            if local.depth > self.locals.depth {
                self.write(OPCODE::POP);
                self.locals.locals.pop();
            } else {
                break;
            }
        }
    }


    fn scope(&mut self) {
        self.locals.depth+=1;// beginScope();
        self.block();
        self.endScope();
    }

    fn consume(&mut self, token_kind: TokenKind, msg: &str) {
        if self.error {return;}
        if self.current().kind == token_kind {
                self.advance();
                return
            } else {
                self.error = true;
                self.error(self.previous().line,
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
        let mut getOp: OPCODE;
        let mut setOp: OPCODE;
        let name = self.previous().lexeme.clone(); // variable name
        
        if let Some(index) = self.resolveLocalVariables(&name) { //if variable is a
            //local variable
            getOp = OPCODE::GET_LOCAL(index); // save the index of the local variable in the opcode
            // associated ith getting and setting local variables
            setOp = OPCODE::SET_LOCAL(index);

        } else { 
            let val = Rc::new(self.previous().lexeme.clone());
            self.write_constant(Value::STRING(val));
            getOp = OPCODE::GET_GLOBAL;
            setOp = OPCODE::SET_GLOBAL;
        }


        if self.match_token(TokenKind::EQUAL) {
            self.expression();
            self.write(setOp);
        } else {
            self.write(getOp)
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
            TokenKind::SLASH | TokenKind::STAR | TokenKind::MOD => PRECEDENCE::Factor,
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL => PRECEDENCE::Comparison,
            _ => PRECEDENCE::None,
        } 
    }


 // returns the precedence level of previous token
    fn getpreviousprecedence(&self) -> PRECEDENCE {
         match self.previous().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR | TokenKind::MOD => PRECEDENCE::Factor,
            _ => PRECEDENCE::None,
        }
 
    }

    fn getinfix(&self) -> Option<fn(&mut Self)> {
         match self.previous().kind {
            TokenKind::MINUS | TokenKind::PLUS | TokenKind::STAR | TokenKind::SLASH |
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL | TokenKind::MOD
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
        if self.error{return;}
        if !self.hadError {
            self.hadError = true;
        }
        self.error = true;
        eprintln!("[line {}]{}", line, message);
    }


    fn parseVariable(&mut self, msg: &str) {
        self.consume(TokenKind::IDENTIFIER, msg);
        let name = self.previous().lexeme.clone();
        if !self.declareLocal(&name) {// if compiler is not within a scope, the var declaration is global
            self.write_constant(Value::STRING(Rc::new(name)));
        } // if compiler is within a scope declare local would have done a lot of things under the
        // hood
    }

    fn varDeclaration(&mut self) {
        self.parseVariable("Expects a variable name");
        if self.match_token(TokenKind::EQUAL) {
            self.expression();
        } else {
            self.write_constant(Value::NIL);
        }

        self.consume(TokenKind::SEMI_COLON, "Syntax error: ';' is expected after declaration");
        if self.locals.depth == 0 { //In globals
            self.write(OPCODE::DEF_GLOBAL);
            return;
        }

        //after the ';' in the defining of local variable is done, then declare the variable defined
        if let Some(s) = self.locals.locals.last_mut() {
            s.depth = self.locals.depth;
        }
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
            if self.previous().kind == TokenKind::SEMI_COLON {return;}
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
            TokenKind::OCURLY => {
                self.advance();
                self.scope();
            }
            _ => self.statement()
        }

        if self.error {self.synchronize();}
    }

    /**
    *   enters a statement
    * */
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
    parser.hadError
}
