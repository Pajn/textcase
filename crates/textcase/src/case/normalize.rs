use crate::{config::SubtitleSeparatorStyle, util::collapse_whitespace};

pub fn normalize_whitespace(input: &str) -> String {
    collapse_whitespace(input)
}

/// Rewrites subtitle separators (`:`, ` - `, ` – `, ` — `) to the requested
/// style.
///
/// A single left-to-right scan replaces each separator in place, so a literal
/// occurrence of the old sentinel text can no longer collide with the marker.
/// A separator is only rewritten when it is flanked by word content and is not
/// a numeric range (`3 - 5`), avoiding false positives on ranges and stray
/// dashes.
pub fn normalize_subtitle_separators(input: &str, style: SubtitleSeparatorStyle) -> String {
    let replacement = match style {
        SubtitleSeparatorStyle::Preserve => return input.to_string(),
        SubtitleSeparatorStyle::ColonSpace => ": ",
        SubtitleSeparatorStyle::SpaceDashSpace => " - ",
        SubtitleSeparatorStyle::EmDashSpace => " — ",
    };

    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut index = 0;
    while index < chars.len() {
        if let Some(consumed) = match_separator(&chars, index) {
            out.push_str(replacement);
            index += consumed;
        } else {
            out.push(chars[index]);
            index += 1;
        }
    }
    out
}

/// Returns the number of characters consumed if a valid subtitle separator
/// starts at `index`.
fn match_separator(chars: &[char], index: usize) -> Option<usize> {
    // " <sep> ": space, separator, space.
    if chars[index] == ' '
        && matches!(chars.get(index + 1), Some('—' | '–' | '-' | ':'))
        && chars.get(index + 2) == Some(&' ')
    {
        return valid_context(prev_char(chars, index), chars.get(index + 3).copied()).then_some(3);
    }

    // ": ": colon, space — only when not already covered by the spaced form
    // above (i.e. not preceded by a space).
    if chars[index] == ':'
        && chars.get(index + 1) == Some(&' ')
        && prev_char(chars, index) != Some(' ')
    {
        return valid_context(prev_char(chars, index), chars.get(index + 2).copied()).then_some(2);
    }

    None
}

fn prev_char(chars: &[char], index: usize) -> Option<char> {
    index.checked_sub(1).and_then(|p| chars.get(p)).copied()
}

/// A separator only splits a title when both sides carry word content, and not
/// when both sides are digits (a numeric range such as `3 - 5`).
fn valid_context(prev: Option<char>, next: Option<char>) -> bool {
    let (Some(prev), Some(next)) = (prev, next) else {
        return false;
    };
    if !prev.is_alphanumeric() || !next.is_alphanumeric() {
        return false;
    }
    !(prev.is_ascii_digit() && next.is_ascii_digit())
}
