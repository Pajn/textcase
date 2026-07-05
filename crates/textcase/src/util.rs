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

pub fn is_all_caps(token: &str) -> bool {
    let mut saw_alpha = false;
    for ch in token.chars() {
        if ch.is_alphabetic() {
            saw_alpha = true;
            if !ch.is_uppercase() {
                return false;
            }
        }
    }
    saw_alpha
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
