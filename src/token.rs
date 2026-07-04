#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenKind {
    BEGIN,
    MINUS, PLUS, STAR, SLASH, INT, DOUBLE, ERROR,
    IDENTIFIER, STRING, CHAR,
    OBRACKET, /*Open Bracket*/ CBRACKET, /*Close Bracket*/
    EOT //end of token
}


pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String
}


impl Token{
    pub fn print_token(&self) {
        println!("Token: {:?}, {}", self.kind, self.lexeme);
    }
}
