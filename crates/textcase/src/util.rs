use sha2::{Digest, Sha256};

pub fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
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
