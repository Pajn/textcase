pub fn is_sentence_terminal(text: &str) -> bool {
    matches!(text, "." | "!" | "?" | "…") || is_wide_sentence_terminal(text)
}

/// Non-Latin sentence terminators (CJK, Arabic, Devanagari). Unlike the ASCII
/// period these never appear in decimals or abbreviations and are not
/// space-separated, so callers treat them as unconditional boundaries.
pub fn is_wide_sentence_terminal(text: &str) -> bool {
    matches!(text, "。" | "！" | "？" | "｡" | "؟" | "।" | "॥")
}

/// How a trailing period after an abbreviation affects sentence detection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbbreviationKind {
    /// Titles and connectives followed by a name or a continuing phrase
    /// ("Dr. Smith", "Kramer vs. Kramer"); the period never ends the sentence.
    Title,
    /// Forms only abbreviated directly before a number ("No. 5", "vol. 2",
    /// "fig. 3"); anywhere else the word is ordinary prose ("the answer is
    /// no.") and the period is a real terminal.
    Numeric,
    /// Forms that can close a phrase or a whole sentence ("Acme Inc.",
    /// "etc."); the period ends the sentence when the input capitalizes the
    /// next word.
    Trailing,
}

const TITLE_ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "st", "sr", "jr", "vs", "feat", "ft", "capt", "sgt", "col",
    "gen", "gov",
];

const NUMERIC_ABBREVIATIONS: &[&str] = &["no", "vol", "fig", "approx"];

const TRAILING_ABBREVIATIONS: &[&str] = &["etc", "co", "inc", "ltd", "dept"];

/// Classifies a lowercased word as a known abbreviation, so callers can decide
/// from context whether a period directly after it ends the sentence.
pub fn abbreviation_kind(word: &str) -> Option<AbbreviationKind> {
    if TITLE_ABBREVIATIONS.contains(&word) {
        Some(AbbreviationKind::Title)
    } else if NUMERIC_ABBREVIATIONS.contains(&word) {
        Some(AbbreviationKind::Numeric)
    } else if TRAILING_ABBREVIATIONS.contains(&word) {
        Some(AbbreviationKind::Trailing)
    } else {
        None
    }
}

/// Returns `true` for a lowercased word that is a known abbreviation of any
/// [`AbbreviationKind`].
pub fn is_abbreviation(word: &str) -> bool {
    abbreviation_kind(word).is_some()
}

pub fn is_subtitle_separator(text: &str) -> bool {
    matches!(text, ":" | "-" | "–" | "—")
}
