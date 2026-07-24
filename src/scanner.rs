use crate::token::{Token, TokenKind};

// struct to scan the strings being read
pub struct Scanner {
    source: String,
    current: usize,
    start: usize,
    line: usize
}

impl Scanner {
    pub fn build(source: String) -> Scanner {
        Scanner {
            source,
            current: 0,
            start: 0,
            line: 1
        }
    }

    fn skip_whitespaces(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                ' ' | '\r' => {
                    self.advance();
                },
                '\n' => {
                    self.advance();
                    self.line += 1;
                }
                '/' => {
                    if self.next_peek() == '/' {
                        while self.peek() != '\n' && !self.end(){
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    // returns the current character yet to be consumed
    fn peek(&mut self) -> char {
        if self.current >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    // returns the character after the current character to be consumed
    fn next_peek(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    // consumes the next character pointed to by self.current
    fn consume(&mut self) -> char {
        // defined variables are mutable bu default
        let character: char;
        if self.current >= self.source.len() {
            character = '\0';
        } else {
            character = self.source.chars().nth(self.current).unwrap_or('\0');
        }
        //self.current += 1;
        self.advance();
        character
    }

    // it consumes the current character without return it
    fn advance(&mut self) {
        self.current += 1;
    }

    // creates a number token
    fn number(&mut self) -> Token{
        let mut decimal = false;
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.next_peek().is_ascii_digit() {
            self.advance();
            decimal = true;
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        if decimal {
            return self.make_token(TokenKind::DOUBLE);
        }

        self.make_token(TokenKind::INT)
    }

    fn identifier(&mut self) -> Token{
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let kind = Scanner::GetTokenKind(&self.source[self.start..self.current]);
        self.make_token(kind)
    }

    fn GetTokenKind(string: &str) -> TokenKind {
        match string {
            "int"   => TokenKind::INT,
            "float" => TokenKind::DOUBLE,
            "char"  => TokenKind::CHAR,
            "string" => TokenKind::STRING,
            "print" => TokenKind::PRINT,
            "false" => TokenKind::FALSE,
            "true"  => TokenKind::TRUE,
            "let"   => TokenKind::LET,
            _       => TokenKind::IDENTIFIER
        }
    }

    // check is the next character is to be consumed is equal the character passed to match
    fn match_character(&mut self, character: char) -> bool {
        character == self.peek()
    }

    fn make_token(&mut self, kind: TokenKind) -> Token{
        Token {
            kind,
            lexeme: self.source[self.start..self.current].to_string(),
            line: self.line
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' {
            self.advance();
        }

        self.advance();
        return Token {
            kind: TokenKind::STRING,
            lexeme: self.source[self.start + 1..self.current - 1].to_string(),
            line: self.line
        };
    }

    fn end(&mut self) -> bool {
        self.current >= self.source.len()
    }

    // > has been consumed already
    fn right_arrow(&mut self) -> Token {
        if self.match_character('=') { // if the next char to be consumed is '='
            self.advance(); // consume it and set the token type for this token to greater than or equal
            // to
            return self.make_token(TokenKind::GREATER_EQUAL);
        }
        self.make_token(TokenKind::GREATER_THAN)
    }


    // < has been consumed already
    fn left_arrow(&mut self) -> Token {
        if self.match_character('=') { // if the next char to be consumed is '='
            self.advance(); // consume it and set the token type for this token to less than or equal
            // to
            return self.make_token(TokenKind::LESS_EQUAL);
        }
        self.make_token(TokenKind::LESS_THAN)
    }

    fn equal_to_sign(&mut self) -> Token{
        if self.match_character('=') {
            self.advance();
            return self.make_token(TokenKind::EQUAL_EQUAL);
        }
        self.make_token(TokenKind::EQUAL)
    }

    fn bang_equal(&mut self) -> Token {
        if self.match_character('=') {
            self.advance();
            return self.make_token(TokenKind::BANG_EQUAL);
        }
        self.make_token(TokenKind::BANG)
    }

    fn character(&mut self) -> Token {
        /*
        * self.current is pointing at the character as the open single quote has been consumed
        */
        self.advance(); // points self.current to the closing single quote
        self.advance(); // points self.current to the next character after tge closing single quote

        self.start += 1;
        Token {
            kind: TokenKind::CHAR,
            lexeme: self.source[self.start..self.current-1].to_string(),
            line: self.line
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespaces();
        self.start = self.current; // after clearing white spaces (if there is any) set the value of

        if self.end() {
            return Token {
                kind: TokenKind::EOT,
                lexeme: "".to_string(),
                line: self.line
            };
        }

        // start to current
        if self.peek().is_ascii_digit() {
            return self.number();
        }

        if self.peek().is_alphabetic() {
            return self.identifier();
        }

        match self.consume() {
            '+' => return self.make_token(TokenKind::PLUS),
            '-' => return self.make_token(TokenKind::MINUS),
            '*' => return self.make_token(TokenKind::STAR),
            '/' => return self.make_token(TokenKind::SLASH),
            '(' => return self.make_token(TokenKind::OBRACKET),
            ')' => return self.make_token(TokenKind::CBRACKET),
            '>' => return self.right_arrow(),
            '<' => return self.left_arrow(),
            '=' => return self.equal_to_sign(),
            '"' => return self.string(),
            '!' => return self.bang_equal(),
            '\'' => return self.character(),
            ';' => return self.make_token(TokenKind::SEMI_COLON),
            _ => return self.make_token(TokenKind::ERROR),
        }
    }
}
