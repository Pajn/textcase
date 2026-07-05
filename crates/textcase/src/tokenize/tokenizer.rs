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
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() {
                    text.push(next);
                    chars.next();
                } else if is_word_connector(next) {
                    // A connector (apostrophe or hyphen) stays inside the word
                    // only when a letter or digit follows it, so trailing quotes
                    // and dashes are emitted as punctuation instead of being
                    // glued onto the word.
                    let mut lookahead = chars.clone();
                    lookahead.next();
                    if lookahead.peek().is_some_and(|c| c.is_alphanumeric()) {
                        text.push(next);
                        chars.next();
                    } else {
                        break;
                    }
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
            || matches!(
                ch,
                '“' | '”' | '„' | '«' | '»' | '…' | '—' | '–'
                // CJK, Arabic and Devanagari terminals and separators.
                | '。' | '！' | '？' | '｡' | '؟' | '।' | '॥' | '、' | '，' | '：'
            )
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

fn is_word_connector(ch: char) -> bool {
    matches!(ch, '\'' | '’' | '-' | '‐' | '‑')
}
