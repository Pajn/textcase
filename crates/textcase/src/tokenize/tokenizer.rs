use super::{Token, TokenKind};

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            let mut text = String::from(ch);
            while let Some(next) = chars.peek() {
                if next.is_whitespace() {
                    text.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                text,
                kind: TokenKind::Whitespace,
            });
            continue;
        }

        if is_word_start(ch) {
            let mut text = String::from(ch);
            while let Some(next) = chars.peek() {
                if is_word_continue(*next) {
                    text.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                text,
                kind: TokenKind::Word,
            });
            continue;
        }

        let kind = if ch.is_ascii_punctuation()
            || matches!(ch, '“' | '”' | '„' | '«' | '»' | '…' | '—' | '–')
        {
            TokenKind::Punctuation
        } else {
            TokenKind::Symbol
        };

        tokens.push(Token {
            text: ch.to_string(),
            kind,
        });
    }

    tokens
}

pub fn reconstruct(tokens: &[Token]) -> String {
    let mut out = String::new();
    for token in tokens {
        out.push_str(&token.text);
    }
    out
}

fn is_word_start(ch: char) -> bool {
    ch.is_alphanumeric()
}

fn is_word_continue(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '\'' | '’' | '-' | '‐' | '‑')
}
