#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Word,
    Whitespace,
    Punctuation,
    Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

impl Token {
    pub fn is_word(&self) -> bool {
        matches!(self.kind, TokenKind::Word)
    }
}
