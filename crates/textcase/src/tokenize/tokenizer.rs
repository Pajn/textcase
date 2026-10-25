use super::{Token, TokenKind};
use unicode_segmentation::UnicodeSegmentation;

pub fn tokenize(input: &str) -> Vec<Token> {
    // Grapheme indices keep combining marks attached to their base while still
    // providing byte offsets into the original input.
    let mut graphemes = input.grapheme_indices(true).peekable();
    let mut tokens = Vec::new();

    while let Some((start, grapheme)) = graphemes.next() {
        let mut end = start + grapheme.len();

        if grapheme.chars().all(char::is_whitespace) {
            while let Some(&(index, next)) = graphemes.peek() {
                if next.chars().all(char::is_whitespace) {
                    end = index + next.len();
                    graphemes.next();
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

        if is_word_grapheme(grapheme) {
            while let Some(&(index, next)) = graphemes.peek() {
                if is_word_grapheme(next) {
                    end = index + next.len();
                    graphemes.next();
                } else if is_word_connector(next) {
                    // A connector (apostrophe or hyphen) stays inside the word
                    // only when a letter or digit follows it, so trailing quotes
                    // and dashes are emitted as punctuation instead of being
                    // glued onto the word.
                    let mut lookahead = graphemes.clone();
                    lookahead.next();
                    if lookahead.peek().is_some_and(|(_, following)| is_word_grapheme(following)) {
                        end = index + next.len();
                        graphemes.next();
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

        let ch = grapheme.chars().next().unwrap_or_default();
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

fn is_word_grapheme(grapheme: &str) -> bool {
    grapheme
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
}

fn is_word_connector(grapheme: &str) -> bool {
    matches!(grapheme, "'" | "’" | "-" | "‐" | "‑")
}
