use crate::tokenize::{Token, TokenKind, is_subtitle_separator};

/// Marks punctuation tokens that act as subtitle separators.
///
/// Mirrors the context rules of `normalize_subtitle_separators`: a separator
/// only opens a subtitle when it is flanked by word content, a colon must be
/// followed by whitespace ("10:30" and "re:invent" stay attached), and
/// numeric or single-letter ranges ("3 - 5", "a - z") do not count.
pub fn subtitle_separator_flags(tokens: &[Token]) -> Vec<bool> {
    let mut flags = vec![false; tokens.len()];
    for index in 0..tokens.len() {
        let token = &tokens[index];
        if !matches!(token.kind, TokenKind::Punctuation) || !is_subtitle_separator(&token.text) {
            continue;
        }
        flags[index] = match token.text.as_str() {
            ":" => colon_is_separator(tokens, index),
            _ => dash_is_separator(tokens, index),
        };
    }
    flags
}

fn colon_is_separator(tokens: &[Token], index: usize) -> bool {
    let followed_by_space = tokens
        .get(index + 1)
        .is_some_and(|next| matches!(next.kind, TokenKind::Whitespace));
    followed_by_space && flanked_by_words(tokens, index)
}

fn dash_is_separator(tokens: &[Token], index: usize) -> bool {
    let spaced =
        is_whitespace_at(tokens, index.wrapping_sub(1)) && is_whitespace_at(tokens, index + 1);
    // An em or en dash may also sit directly between words ("Title—Subtitle");
    // an unspaced ASCII hyphen never reaches here because the tokenizer keeps
    // it inside the word.
    let attached = tokens[index].text != "-"
        && index > 0
        && tokens[index - 1].is_word()
        && tokens.get(index + 1).is_some_and(Token::is_word);
    (spaced || attached) && flanked_by_words(tokens, index)
}

/// Both neighbors (skipping whitespace) must be words, and the pair must not
/// form a range: two numbers ("3 - 5") or two single characters ("a - z").
fn flanked_by_words(tokens: &[Token], index: usize) -> bool {
    let previous = tokens[..index]
        .iter()
        .rev()
        .find(|token| !matches!(token.kind, TokenKind::Whitespace));
    let next = tokens[index + 1..]
        .iter()
        .find(|token| !matches!(token.kind, TokenKind::Whitespace));
    let (Some(previous), Some(next)) = (previous, next) else {
        return false;
    };
    if !previous.is_word() || !next.is_word() {
        return false;
    }
    !is_range_pair(&previous.text, &next.text)
}

fn is_whitespace_at(tokens: &[Token], index: usize) -> bool {
    tokens
        .get(index)
        .is_some_and(|token| matches!(token.kind, TokenKind::Whitespace))
}

fn is_range_pair(previous: &str, next: &str) -> bool {
    let all_digits = |word: &str| word.chars().all(|ch| ch.is_ascii_digit());
    let single = |word: &str| word.chars().count() == 1;
    (all_digits(previous) && all_digits(next)) || (single(previous) && single(next))
}
