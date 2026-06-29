use crate::token::{Token, TokenKind};

// struct to scan the strings being read
pub struct Scanner<'a> {
    source: &'a str,
    current: usize,
    start: usize,
}

impl<'a> Scanner<'a> {
    pub fn build(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            current: 0,
            start: 0,
        }
    }

    fn skip_whitespaces(&mut self) {
        loop {
            let ch = self.peek();
            match ch {
                ' ' | '\n' | '\r' => self.current += 1,
                '/' => {
                    if self.next_peek() == '/' {
                        'comment: for i in self.source.as_bytes()[self.current..].iter() {
                            match i {
                                b'\n' => {
                                    self.advance();
                                    break 'comment;
                                }
                                _ => self.advance(),
                            }
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
        self.current += 1;
        character
    }

    // it consumes the current character without return it
    fn advance(&mut self) {
        self.current += 1;
    }

    // creates a number token
    fn number(&mut self) -> Token <'_>{
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

    fn identifier(&mut self) -> Token <'_>{
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        self.make_token(TokenKind::IDENTIFIER)
    }

    // check is the next character is to be consumed is equal the character passed to match
    fn match_character(&mut self, character: char) -> bool {
        character == self.peek()
    }

    fn make_token(&mut self, kind: TokenKind) -> Token <'_>{
        Token {
            kind,
            lexeme: &self.source[self.start..self.current],
        }
    }

    fn string(&mut self) -> Token<'_> {
        while self.peek() != '"' {
            self.advance();
        }

        self.advance();
        return self.make_token(TokenKind::STRING)
    }

    pub fn scan_token(&mut self) -> Token<'_> {
        self.skip_whitespaces();
        self.start = self.current; // after clearing white spaces (if there is any) set the value of

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
            '*' => return self.make_token(TokenKind::SLASH),
            '/' => return self.make_token(TokenKind::SLASH),
            '(' => return self.make_token(TokenKind::OBRACKET),
            ')' => return self.make_token(TokenKind::CBRACKET),
            '"' => return self.string(),
            _ => return self.make_token(TokenKind::ERROR),
        }
    }
}
