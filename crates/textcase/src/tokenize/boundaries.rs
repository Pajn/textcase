pub fn is_sentence_terminal(text: &str) -> bool {
    matches!(text, "." | "!" | "?" | "…") || is_wide_sentence_terminal(text)
}

/// Non-Latin sentence terminators (CJK, Arabic, Devanagari). Unlike the ASCII
/// period these never appear in decimals or abbreviations and are not
/// space-separated, so callers treat them as unconditional boundaries.
pub fn is_wide_sentence_terminal(text: &str) -> bool {
    matches!(text, "。" | "！" | "？" | "｡" | "؟" | "।" | "॥")
}

/// Common abbreviations whose trailing period does not end a sentence.
const ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "st", "sr", "jr", "vs", "etc", "no", "vol", "feat", "ft",
    "approx", "dept", "fig", "co", "inc", "ltd", "gen", "gov", "capt", "sgt", "col",
];

/// Returns `true` for a lowercased word that is a known abbreviation, so a
/// period directly after it is not treated as a sentence boundary.
pub fn is_abbreviation(word: &str) -> bool {
    ABBREVIATIONS.contains(&word)
}

pub fn is_subtitle_separator(text: &str) -> bool {
    matches!(text, ":" | "-" | "–" | "—")
}
