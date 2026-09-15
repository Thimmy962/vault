#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub enum TokenKind {
    PRINT, SEMI_COLON, LET,
    MINUS, PLUS, STAR, SLASH, INT, MOD, DOUBLE, ERROR,
    IDENTIFIER, STRING, CHAR,
    GREATER_EQUAL, LESS_EQUAL, 
    LESS_THAN, GREATER_THAN,
    EQUAL, EQUAL_EQUAL,
    BANG, BANG_EQUAL,
    FALSE, TRUE,
    WHILE, FOR, //loops
    AND, OR, IF, ELSE,
    OBRACKET, /*Open Bracket*/ CBRACKET, /*Close Bracket*/
    OCURLY, /*Open curly braces*/ CCURLY, /*Close curly braces*/
    EOT //end of token
}


pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize
}


impl Token{
    pub fn print_token(&self) {
        println!("Token: {:?}, {}", self.kind, self.lexeme);
    }
}
