use super::{Token, TokenKind};

pub fn tokenize(input: &str) -> Vec<Token> {
    // `char_indices` gives the byte offset of each char, so every token can
    // record its range in the raw input; `end` tracks the byte just past the
    // last char consumed for the current token.
    let mut chars = input.char_indices().peekable();
    let mut tokens = Vec::new();

    while let Some((start, ch)) = chars.next() {
        let mut end = start + ch.len_utf8();

        if ch.is_whitespace() {
            while let Some(&(index, next)) = chars.peek() {
                if next.is_whitespace() {
                    end = index + next.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                text: input[start..end].to_string(),
                kind: TokenKind::Whitespace,
                source: start..end,
            });
            continue;
        }

        if is_word_start(ch) {
            while let Some(&(index, next)) = chars.peek() {
                if next.is_alphanumeric() {
                    end = index + next.len_utf8();
                    chars.next();
                } else if is_word_connector(next) {
                    // A connector (apostrophe or hyphen) stays inside the word
                    // only when a letter or digit follows it, so trailing quotes
                    // and dashes are emitted as punctuation instead of being
                    // glued onto the word.
                    let mut lookahead = chars.clone();
                    lookahead.next();
                    if lookahead.peek().is_some_and(|(_, c)| c.is_alphanumeric()) {
                        end = index + next.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            tokens.push(Token {
                text: input[start..end].to_string(),
                kind: TokenKind::Word,
                source: start..end,
            });
            continue;
        }

        let kind = if ch.is_ascii_punctuation()
            || matches!(
                ch,
                '“' | '”' | '„' | '«' | '»' | '…' | '—' | '–'
                // CJK, Arabic and Devanagari terminals and separators.
                | '。' | '！' | '？' | '｡' | '؟' | '।' | '॥' | '、' | '，' | '：'
            ) {
            TokenKind::Punctuation
        } else {
            TokenKind::Symbol
        };

        tokens.push(Token {
            text: input[start..end].to_string(),
            kind,
            source: start..end,
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
