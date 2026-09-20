use std::rc::Rc;

use crate::bytes::{Bytes, Value, OPCODE};
use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};

/// Defines the operator precedence levels for Pratt parsing in ascending order.
#[derive(PartialEq, Clone, Copy, Debug, PartialOrd, Ord, Eq)]
pub enum PRECEDENCE {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // ==
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl PRECEDENCE {
    /// Returns the next highest precedence level in the operator hierarchy.
    pub fn next(&self) -> Self {
        match self {
            PRECEDENCE::None => PRECEDENCE::Assignment,
            PRECEDENCE::Assignment => PRECEDENCE::Or,
            PRECEDENCE::Or => PRECEDENCE::And,
            PRECEDENCE::And => PRECEDENCE::Equality,
            PRECEDENCE::Equality => PRECEDENCE::Comparison,
            PRECEDENCE::Comparison => PRECEDENCE::Term,
            PRECEDENCE::Term => PRECEDENCE::Factor,
            PRECEDENCE::Factor => PRECEDENCE::Unary,
            PRECEDENCE::Unary => PRECEDENCE::Call,
            PRECEDENCE::Call => PRECEDENCE::Primary,
            PRECEDENCE::Primary => PRECEDENCE::Primary,
        }
    }
}

/// Represents a single local variable record in the current scope stack.
pub struct Local {
    name: String, // Variable identifier name
    depth: i8,    // Lexical scope depth level where variable was declared (-1 = uninitialized)
}

impl Local {
    /// Creates a new `Local` variable record initialized to uninitialized depth (`-1`).
    pub fn new(name: String) -> Self {
        Local { depth: -1, name }
    }
}

/// Tracks the active local variables and current scope depth during compilation.
pub struct Locals {
    locals: Vec<Local>, // Stack of active local variable declarations
    depth: i8,          // Current lexical scope depth level
}

impl Locals {
    /// Initializes a new, empty scope tracker for local variables at global depth (`0`).
    pub fn new() -> Self {
        Locals {
            locals: Vec::new(),
            depth: 0,
        }
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
pub struct Parser<'a> {
    bytecodes: &'a mut Bytes, // Buffer where emitted bytecode instructions are written
    scanner: &'a mut Scanner, // Token stream source
    previous: Option<Token>,  // Previously processed token
    current: Option<Token>,   // Currently lookahead token
    locals: Locals,           // Local variable symbol table and depth tracker
    error: bool,              // Transient panic-mode error flag
    hadError: bool,           // Persistent error flag indicating overall compilation failure
}

impl<'a> Parser<'a> {
    /// Appends a new local variable to the scope tracker.
    fn addLocal(locals: &mut Locals, name: String) {
        // Push a new uninitialized Local record onto the local variable stack
        locals.locals.push(Local::new(name));
    }

    /// Resolves a local variable by name starting from the innermost scope outward.
    ///
    /// Returns `Some(stack_index)` if found in the current lexical block, or `None` if non-local.
    fn resolveLocalVariables(&mut self, name: &str) -> Option<usize> {
        let line = self.previous().line;
        // Search local variables backward to find the most tightly scoped matching name
        for (index, val) in self.locals.locals.iter().rev().enumerate() {
            if val.name == name {
                // Prevent reading a local variable inside its own initializer expression
                if val.depth == -1 {
                    self.error(line, "Can't read local variable for its own initialization".to_string());
                }
                return Some(index);
            }
        }
        None
    }

    /// Returns a reference to the previous token.
    fn previous(&self) -> &Token {
        return self.previous.as_ref().unwrap();
    }

    /// Returns a reference to the current token.
    fn current(&self) -> &Token {
        return self.current.as_ref().unwrap();
    }

    /// Writes a constant value to the bytecode stream.
    fn write_constant(&mut self, value: Value) {
        self.bytecodes.write_constant(value);
    }

    /// Writes an instruction opcode without operand data to the bytecode stream.
    fn write(&mut self, code: OPCODE) {
        self.bytecodes.write(code);
    }

    /// Moves current token to previous slot and scans a new valid token.
    fn advance(&mut self) {
        // Shift current token to previous token slot
        self.previous = self.current.take();
        'advance: loop {
            // Fetch next token from scanner
            self.current = Some(self.scanner.scan_token());
            // Skip lexical errors and keep scanning until a valid token is retrieved
            if self.current().kind != TokenKind::ERROR {
                break 'advance;
            }
        }
    }

    /// Used to handle unary tokens like `!` (bang) and `-` (negate).
    fn unary(&mut self) {
        let opcode: OPCODE;
        // Map token kind to corresponding unary opcode
        match self.previous().kind {
            TokenKind::MINUS => opcode = OPCODE::NEGATE,
            TokenKind::BANG => opcode = OPCODE::BANG,
            _ => return,
        }

        // Parse operand with unary precedence level
        self.parseprecedence(PRECEDENCE::Unary);

        // Emit corresponding unary operation bytecode
        self.bytecodes.write(opcode);
    }

    /// Parses binary expressions, determining precedence and emitting arithmetic or comparison opcodes.
    fn binary(&mut self) {
        let opcode: OPCODE;
        // Map operator token to binary opcode
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
            _ => return,
        }

        // Parse right hand operand with higher precedence level to maintain precedence rules
        let prec = self.getpreviousprecedence();
        self.parseprecedence(prec.next());
        self.bytecodes.write(opcode);
    }

    /// Parses numeric literal tokens (integers or floating-point numbers) into bytecode constants.
    fn number(&mut self) {
        if self.previous().kind == TokenKind::INT {
            // Parse and write 32-bit signed integer value constant
            let val = self.previous().lexeme.clone().parse::<i32>().unwrap();
            self.write_constant(Value::INT(val));
        } else {
            // Parse and write 32-bit floating point value constant
            let val = self.previous().lexeme.clone().parse::<f32>().unwrap();
            self.write_constant(Value::DOUBLE(val));
        }
    }

    /// Registers a local variable name in the current scope if within a non-global block context.
    fn declareLocal(&mut self, name: &str) -> bool {
        // Global scope (depth 0) declarations are handled separately as globals
        if self.locals.depth == 0 {
            return false;
        }
        let name_str = name.to_string();
        let mut isDuplicate = false;

        // Verify variable is not already declared within the current scope level
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
        // Register variable name into active scope local list
        Parser::addLocal(&mut self.locals, name_str);
        true
    }

    /// Checks if current token matches expected `TokenKind`.
    fn check(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    /// Parses declarations contained within a block scope until an end-brace or EOF token is met.
    fn block(&mut self) {
        // Parse statements inside block until closing brace or EOF token is encountered
        while !self.check(TokenKind::CCURLY) && !self.check(TokenKind::EOT) {
            self.declaration();
        }

        self.consume(TokenKind::CCURLY, "Expects '}' to end this scope");
    }

    /// Exits the current local scope depth and pops out-of-scope stack variables from the local environment.
    fn endScope(&mut self) {
        // Decrement active lexical depth tracker
        self.locals.depth -= 1;
        // Emit POP opcodes for all variables defined within the exited scope level
        while let Some(local) = self.locals.locals.last() {
            if local.depth > self.locals.depth {
                self.write(OPCODE::POP);
                self.locals.locals.pop();
            } else {
                break;
            }
        }
    }

    /// Enters, parses, and exits a scoped block environment.
    fn scope(&mut self) {
        self.locals.depth += 1; // Increment current lexical scope depth
        self.block();
        self.endScope();
    }

    /// Asserts that the current token matches the expected kind, advancing past it or flagging a syntax error.
    fn consume(&mut self, token_kind: TokenKind, msg: &str) {
        if self.error {
            return;
        }
        if self.check(token_kind) {
            self.advance();
            return;
        } else {
            self.error = true;
            self.error(self.previous().line, msg.to_string());
        }
    }

    /// Parses primitive literals (booleans, strings, and characters) into bytecode constants.
    fn literal(&mut self) {
        match self.previous().kind {
            TokenKind::CHAR => {
                // Parse single character string literal
                let val = self.previous().lexeme.clone();
                let val = val.chars().next().unwrap();
                self.write_constant(Value::CHAR(val));
            }
            TokenKind::STRING => {
                // Wrap heap-allocated string literal in reference counting pointer
                let val = self.previous().lexeme.clone();
                let a = Rc::new(val);
                self.write_constant(Value::STRING(a));
            }
            TokenKind::FALSE => self.write_constant(Value::BOOL(false)),
            TokenKind::TRUE => self.write_constant(Value::BOOL(true)),
            _ => return,
        }
    }

    /// Handles identifier token evaluation for reading or assigning local and global variables.
    fn variable(&mut self) {
        let mut getOp: OPCODE;
        let mut setOp: OPCODE;
        let name = self.previous().lexeme.clone(); // Capture variable identifier lexeme

        // Determine variable context: local frame stack vs global variable map
        if let Some(index) = self.resolveLocalVariables(&name) {
            getOp = OPCODE::GET_LOCAL(index);
            setOp = OPCODE::SET_LOCAL(index);
        } else {
            let val = Rc::new(self.previous().lexeme.clone());
            self.write_constant(Value::STRING(val));
            getOp = OPCODE::GET_GLOBAL;
            setOp = OPCODE::SET_GLOBAL;
        }

        // Branch logic to differentiate between assignment expressions and variable access
        if self.match_token(TokenKind::EQUAL) {
            self.expression();
            self.write(setOp);
        } else {
            self.write(getOp);
        }
    }

    /// Handles grouped bracketed expressions `( ... )`.
    fn bracket(&mut self) {
        self.parseprecedence(PRECEDENCE::Assignment);
        self.consume(TokenKind::CBRACKET, "Expects a ')'");
    }

    /// Gets a prefix parsing function for the previous token, or `None` if no prefix rule exists.
    fn getprefix(&self) -> Option<fn(&mut Self)> {
        match self.previous().kind {
            TokenKind::MINUS | TokenKind::BANG => Some(Self::unary),
            TokenKind::INT | TokenKind::DOUBLE => Some(Self::number),
            TokenKind::STRING | TokenKind::CHAR | TokenKind::FALSE | TokenKind::TRUE => Some(Self::literal),
            TokenKind::OBRACKET => Some(Self::bracket),
            TokenKind::IDENTIFIER => Some(Self::variable),
            _ => None,
        }
    }

    /// Returns the precedence level of the current token.
    fn getcurrentprecedence(&self) -> PRECEDENCE {
        match self.current().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR | TokenKind::MOD => PRECEDENCE::Factor,
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL => PRECEDENCE::Comparison,
            TokenKind::AND => PRECEDENCE::And,
            TokenKind::OR => PRECEDENCE::Or,
            TokenKind::EQUAL => PRECEDENCE::Assignment,
            _ => PRECEDENCE::None,
        }
    }

    /// Returns the precedence level of the previous token.
    fn getpreviousprecedence(&self) -> PRECEDENCE {
        match self.previous().kind {
            TokenKind::MINUS | TokenKind::PLUS => PRECEDENCE::Term,
            TokenKind::SLASH | TokenKind::STAR | TokenKind::MOD => PRECEDENCE::Factor,
            _ => PRECEDENCE::None,
        }
    }

    /// Gets an infix parsing function for the previous token, or `None` if no infix rule exists.
    fn getinfix(&self) -> Option<fn(&mut Self)> {
        match self.previous().kind {
            TokenKind::MINUS | TokenKind::PLUS | TokenKind::STAR | TokenKind::SLASH |
            TokenKind::GREATER_THAN | TokenKind::LESS_THAN | TokenKind::EQUAL_EQUAL |
            TokenKind::GREATER_EQUAL | TokenKind::LESS_EQUAL | TokenKind::BANG_EQUAL | TokenKind::MOD => Some(Self::binary),

            TokenKind::AND => Some(Self::infix_and),
            TokenKind::OR => Some(Self::infix_or),
            _ => None,
        }
    }

    /// Used to handle logical `&&` short-circuiting operator in comparison expressions.
    fn infix_and(&mut self) {
        todo!();
    }

    /// Used to handle logical `||` short-circuiting operator in comparison expressions.
    fn infix_or(&mut self) {
        todo!();
    }

    /// Reports a compilation error, setting error flags and printing the diagnostic message.
    fn error(&mut self, line: usize, message: String) {
        if self.error {
            return;
        }
        if !self.hadError {
            self.hadError = true;
        }
        self.error = true;
        eprintln!("[line {}]{}", line, message);
    }

    /// Consumes a variable identifier token and declares it as local or pre-allocates global reference.
    fn parseVariable(&mut self, msg: &str) {
        self.consume(TokenKind::IDENTIFIER, msg);
        let name = self.previous().lexeme.clone();
        // If not inside a scope, register variable as global symbol constant
        if !self.declareLocal(&name) {
            self.write_constant(Value::STRING(Rc::new(name)));
        }
    }

    /// Parses a variable statement declaration (e.g., `let x = 5;`).
    fn varDeclaration(&mut self) {
        self.parseVariable("Expects a variable name");
        if self.match_token(TokenKind::EQUAL) {
            self.expression();
        } else {
            // Default uninitialized variables to Value::NIL
            self.write_constant(Value::NIL);
        }

        self.consume(TokenKind::SEMI_COLON, "Syntax error: ';' is expected after declaration");
        if self.locals.depth == 0 {
            // Write global definition opcode for top-level variables
            self.write(OPCODE::DEF_GLOBAL);
            return;
        }

        // Mark variable as initialized by updating its depth to the active depth
        if let Some(s) = self.locals.locals.last_mut() {
            s.depth = self.locals.depth;
        }
    }

    /// Synchronizes the parser state after hitting a syntax error to recover at statement boundaries.
    fn synchronize(&mut self) {
        self.error = false;
        while !self.match_token(TokenKind::EOT) {
            if self.previous().kind == TokenKind::SEMI_COLON {
                return;
            }
            match self.current().kind {
                TokenKind::SEMI_COLON | TokenKind::LET | TokenKind::STRING | TokenKind::PRINT |
                TokenKind::INT | TokenKind::DOUBLE | TokenKind::CHAR => break,
                _ => self.advance(),
            }
        }
    }

    /// Updates a previous jump placeholder instruction with the calculated jump distance offset.
    fn patchJump(&mut self, index: usize) {
        let jump = self.bytecodes.length() - index;
        if let Some(val) = self.bytecodes.read_bytecode_based_on_index(index) {
            match val {
                OPCODE::JUMP_IF_FALSE(_) => self.bytecodes.write_bytecode_based_on_index(index, OPCODE::JUMP_IF_FALSE(jump - 1)),
                OPCODE::JUMP(_) => self.bytecodes.write_bytecode_based_on_index(index, OPCODE::JUMP(jump - 1)),
                _ => return,
            }
        }
    }

    /// Emits a backward jump instruction targeting a specified bytecode offset.
    fn patchLoop(&mut self, index: usize) {
        let offset = self.bytecodes.length() - index + 1;
        self.write(OPCODE::WHILEJUMP(offset));
    }

    /// Compiles `if` conditional statements with optional `else` branches.
    fn ifStatement(&mut self) {
        // Parse condition expression inside parentheses: '(' <expression> ')'
        self.expect_token(TokenKind::OBRACKET, "Expected ( after if statement condition.");
        self.expression();
        self.expect_token(TokenKind::CBRACKET, "Expected ( after the expression in the if statement condition.");

        // Write conditional jump placeholder to skip 'then' block if expression evaluates false
        self.bytecodes.write(OPCODE::JUMP_IF_FALSE(0));

        // Save index of JUMP_IF_FALSE instruction to backpatch jump distance later
        let thenJump = self.bytecodes.length() - 1;

        // Pop condition value from evaluation stack before executing body
        self.write(OPCODE::POP);
        self.statement();

        // Write placeholder unconditional jump to bypass 'else' branch on completion of 'then' block
        self.bytecodes.write(OPCODE::JUMP(0));
        let jump = self.bytecodes.length() - 1;

        // Patch initial JUMP_IF_FALSE offset to jump directly to start of 'else' block or end of 'if' statement
        self.patchJump(thenJump);
        self.write(OPCODE::POP);

        // Compile 'else' block if present
        if self.match_token(TokenKind::ELSE) {
            self.statement();
        }
        // Patch unconditional jump offset to point past the 'else' block
        self.patchJump(jump);
    }

    /// Compiles `while` loop structures.
    fn whileStatement(&mut self) {
        // Store starting instruction index of loop condition check
        let offset = self.bytecodes.length();
        self.expect_token(TokenKind::OBRACKET, "Expected ( after if statement condition.");
        self.expression();
        self.expect_token(TokenKind::CBRACKET, "Expected ) after the expression in the if statement condition.");

        // Emit placeholder jump to exit loop when condition evaluates false
        self.bytecodes.write(OPCODE::JUMP_IF_FALSE(0));
        let thenJump = self.bytecodes.length() - 1;

        // Pop loop condition evaluation result from evaluation stack
        self.bytecodes.write(OPCODE::POP);
        self.statement();

        // Emit backward jump back to condition check offset
        self.patchLoop(offset);

        // Patch false condition jump to jump past loop body on termination
        self.patchJump(thenJump);
        self.bytecodes.write(OPCODE::POP);
    }

    /// Compiles `for` loop constructs into equivalent lower-level jump bytecodes.
    fn forStatement(&mut self) {
        // Create new scope level for variables declared within loop initializer
        self.locals.depth += 1;
        self.expect_token(TokenKind::OBRACKET, "Expected ( after for.");

        // 1. Initializer Clause: handle semicolon (empty), 'let' declaration, or expression statement
        match self.current().kind {
            TokenKind::SEMI_COLON => { self.advance(); }
            TokenKind::LET => {
                self.advance();
                self.varDeclaration();
            }
            _ => self.expressionStatement(),
        }

        let mut loopStart = self.bytecodes.length();
        let mut exitJump: i32 = -1;

        // 2. Condition Clause: optional conditional expression
        if self.current().kind != TokenKind::SEMI_COLON {
            self.expression();
            self.consume(TokenKind::SEMI_COLON, "Syntax error: ';' is expected after declaration");

            // Emit conditional jump instruction to exit loop when condition is false
            self.bytecodes.write(OPCODE::JUMP_IF_FALSE(0));
            exitJump = (self.bytecodes.length() - 1) as i32;

            self.bytecodes.write(OPCODE::POP);
        } else {
            self.advance();
        }

        // 3. Increment/Decrement Clause: optional loop step update
        if self.current().kind != TokenKind::CBRACKET {
            // Jump over increment expression block directly to main loop body during normal iteration
            self.bytecodes.write(OPCODE::JUMP(0));
            let opJump = self.bytecodes.length() - 1;

            let increment_decrement_start = self.bytecodes.length();
            self.expression();
            self.expect_token(TokenKind::CBRACKET, "Expected ) after for statement clause.");

            // Loop back to condition test following increment execution
            self.patchLoop(loopStart);
            loopStart = increment_decrement_start;
            self.patchJump(opJump);
        } else {
            self.advance();
        }

        // Compile main body statement inside loop
        self.statement();
        // Emit loop back jump to increment block or condition evaluation target
        self.patchLoop(loopStart);

        // Patch exit jump destination if condition expression was specified
        if exitJump != -1 {
            self.patchJump(exitJump as usize);
        }
        // Discard local loop scope and clean up stack frame variables
        self.endScope();
    }

    /// Enters and parses statement structures.
    fn statement(&mut self) {
        match self.current().kind {
            TokenKind::PRINT => {
                self.advance();
                self.printstatement();
            }
            TokenKind::IF => {
                self.advance();
                self.ifStatement();
            }
            TokenKind::WHILE => {
                self.advance();
                self.whileStatement();
            }
            TokenKind::FOR => {
                self.advance();
                self.forStatement();
            }
            TokenKind::OCURLY => {
                self.advance();
                self.scope();
            }
            _ => {
                self.expressionStatement();
            }
        }
    }

    /// Parses an expression statement and emits a POP bytecode instruction.
    fn expressionStatement(&mut self) {
        self.expression();
        self.consume(TokenKind::SEMI_COLON, "Expect ';' after expression.");
        self.write(OPCODE::POP);
    }

    /// Validates expected token kind placeholder.
    fn expect_token(&mut self, token: TokenKind, msg: &str) {
        self.consume(token, msg);
    }

    /// Parses print statements and emits print opcodes.
    fn printstatement(&mut self) {
        self.expression();
        self.consume(TokenKind::SEMI_COLON, "Expects ';' after value");
        self.write(OPCODE::PRINT);
    }

    /// Builds a new `Parser` instance tied to the provided scanner and bytecode buffer.
    pub fn build(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes) -> Self {
        Parser {
            scanner,
            bytecodes,
            previous: None,
            current: None,
            error: false,
            hadError: false,
            locals: Locals::new(),
        }
    }

    /// Parses an expression starting at assignment precedence level.
    pub fn expression(&mut self) {
        self.parseprecedence(PRECEDENCE::Assignment);
    }

    /// Finalizes compilation by consuming the EOF token and writing the end-of-tape instruction opcode.
    pub fn endcompiler(&mut self) {
        self.consume(TokenKind::EOT, "Expects end of token");
        self.write(OPCODE::EOT);
    }

    /// Checks the current token against an expected token kind and advances if matched.
    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if !(self.current().kind == kind) {
            return false;
        }

        self.advance();
        true
    }

    /// Parses top-level statements or declarations (variable declarations, block scopes, or expressions).
    pub fn declaration(&mut self) {
        match self.current().kind {
            TokenKind::LET => {
                self.advance();
                self.varDeclaration();
            }
            _ => self.statement(),
        }

        if self.error {
            self.synchronize();
        }
    }

    /// Parses expressions using a Pratt precedence parsing scheme according to precedence levels.
    pub fn parseprecedence(&mut self, precedence: PRECEDENCE) {
        self.advance(); // Advance current token to previous token slot
        match self.getprefix() {
            Some(func) => func(self),
            None => {
                self.error(
                    self.previous().line,
                    "Syntax error: '".to_owned() + &self.previous().lexeme + "' is not a valid syntax",
                );
                return;
            }
        }

        // Loop and evaluate higher precedence infix operations
        while self.current().kind != TokenKind::EOT && self.getcurrentprecedence() >= precedence {
            self.advance();
            match self.getinfix() {
                Some(func) => func(self),
                None => {
                    self.error(
                        self.previous().line,
                        "Syntax error: '".to_owned() + &self.previous().lexeme + "' is not a valid syntax",
                    );
                    return;
                }
            }
        }
    }
}

/// Compiles source tokens into bytecode, returning true if any compilation errors occurred.
pub fn compile<'a>(scanner: &'a mut Scanner, bytecodes: &'a mut Bytes) -> bool {
    let mut parser = Parser::build(scanner, bytecodes);
    parser.advance();

    // Iterate through input token stream compiling statements until End-Of-Tape
    while !parser.match_token(TokenKind::EOT) {
        parser.declaration();
    }

    parser.endcompiler();
    parser.hadError
}
