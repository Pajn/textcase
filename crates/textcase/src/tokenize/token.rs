#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Word,
    Whitespace,
    Punctuation,
    Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// The token's text. Mutated in place as the input is recased and
    /// normalized; [`Token::source`] stays fixed to the original bytes.
    pub text: String,
    pub kind: TokenKind,
    /// Byte range of the token in the raw input, before any normalization or
    /// recasing. Stable across text rewrites, so it maps a token back to what
    /// the caller passed in.
    pub source: std::ops::Range<usize>,
}

impl Token {
    pub fn is_word(&self) -> bool {
        matches!(self.kind, TokenKind::Word)
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self.kind, TokenKind::Whitespace)
    }
}
