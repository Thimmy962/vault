#[derive(Debug)]
pub enum TokenKind {
    MINUS, PLUS, STAR, SLASH, INT, DOUBLE, ERROR,
    OBRACKET, /*Open Bracket*/ CBRACKET, /*Close Bracket*/
}


pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub length: u8
}


impl Token {
    pub fn print_token(&self) {
        println!("Token: {:?}", self.kind);
    }
}
