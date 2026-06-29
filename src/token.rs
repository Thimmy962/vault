#[derive(Debug)]
pub enum TokenKind {
    MINUS, PLUS, STAR, SLASH, INT, DOUBLE, ERROR,
    IDENTIFIER, STRING,
    OBRACKET, /*Open Bracket*/ CBRACKET, /*Close Bracket*/
}


pub struct Token <'a>{
    pub kind: TokenKind,
    pub lexeme: &'a str
}


impl <'a>Token <'a>{
    pub fn print_token(&self) {
        println!("Token: {:?}, {}", self.kind, self.lexeme);
    }
}
