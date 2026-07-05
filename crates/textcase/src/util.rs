use sha2::{Digest, Sha256};

pub fn collapse_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_whitespace = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !out.is_empty() && !in_whitespace {
                out.push(' ');
            }
            in_whitespace = true;
        } else {
            out.push(ch);
            in_whitespace = false;
        }
    }
    out.trim().to_string()
}

/// Collapses runs of whitespace like [`collapse_whitespace`] but keeps line
/// breaks: a whitespace run containing a newline collapses to a single `\n`,
/// any other run to a single space. Leading and trailing whitespace is trimmed.
pub fn collapse_whitespace_preserving_newlines(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_whitespace = false;
    let mut run_has_newline = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            in_whitespace = true;
            run_has_newline |= ch == '\n';
        } else {
            if in_whitespace && !out.is_empty() {
                out.push(if run_has_newline { '\n' } else { ' ' });
            }
            out.push(ch);
            in_whitespace = false;
            run_has_newline = false;
        }
    }
    out.trim().to_string()
}

pub fn normalize_lookup_key(input: &str) -> String {
    collapse_whitespace(input).to_lowercase()
}

pub fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn lowercase_ascii_words(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(normalize_lookup_key)
        .filter(|part| !part.is_empty())
        .collect()
}

/// Returns `true` for a token that could be an acronym: at least two letters,
/// all uppercase. Single letters ("A") and any word with a lowercase letter
/// are excluded, so a stray capital or a shouted ordinary word is not mistaken
/// for an acronym.
pub fn is_acronym_candidate(token: &str) -> bool {
    let mut letters = 0usize;
    for ch in token.chars() {
        if ch.is_alphabetic() {
            if !ch.is_uppercase() {
                return false;
            }
            letters += 1;
        }
    }
    letters >= 2
}

/// Returns `true` when the whole text is written in capitals (a shouting title)
/// rather than merely containing isolated acronyms: it has at least one
/// uppercase letter and no lowercase ones.
pub fn is_shouting(text: &str) -> bool {
    let mut saw_upper = false;
    for ch in text.chars() {
        if ch.is_lowercase() {
            return false;
        }
        if ch.is_uppercase() {
            saw_upper = true;
        }
    }
    saw_upper
}

/// Returns `true` for tokens with an internal capital, such as `iPhone`,
/// `McDonald`, or `LaTeX`.
///
/// A leading capital followed only by lowercase (an ordinary title-cased word
/// like `Quick`) is *not* mixed case, otherwise preserving mixed case would
/// turn the converter into a no-op on already-capitalized input.
pub fn is_mixed_case(token: &str) -> bool {
    let mut saw_lower = false;
    let mut saw_upper_after_first_alpha = false;
    let mut seen_alpha = false;
    for ch in token.chars() {
        if ch.is_uppercase() {
            if seen_alpha {
                saw_upper_after_first_alpha = true;
            }
        } else if ch.is_lowercase() {
            saw_lower = true;
        }
        if ch.is_alphabetic() {
            seen_alpha = true;
        }
    }
    saw_lower && saw_upper_after_first_alpha
}
