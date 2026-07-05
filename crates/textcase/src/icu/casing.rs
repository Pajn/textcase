use icu_casemap::{CaseMapper, TitlecaseMapper, options::TitlecaseOptions};
use icu_locale_core::LanguageIdentifier;
use unicode_segmentation::UnicodeSegmentation;

pub fn lowercase_locale(input: &str, locale: &str) -> String {
    CaseMapper::new()
        .lowercase_to_string(input, &locale_id(locale))
        .into_owned()
}

pub fn capitalize_word_locale(input: &str, locale: &str) -> String {
    titlecase_word_locale(input, locale)
}

/// Title-cases a single word, capitalizing the first letter of each segment.
///
/// Segments are split on hyphens and on an apostrophe that follows a
/// single-letter prefix (`O'Brien`); an apostrophe inside a contraction
/// (`don't`) stays within its segment. Each segment is title-cased through ICU,
/// so locale rules apply — e.g. Dutch `ijssel` becomes `IJssel`.
pub fn titlecase_word_locale(input: &str, locale: &str) -> String {
    let mapper = TitlecaseMapper::new();
    let id = locale_id(locale);
    let options = TitlecaseOptions::default();

    let mut out = String::with_capacity(input.len());
    let mut segment = String::new();
    let mut letters_in_segment = 0usize;

    for grapheme in UnicodeSegmentation::graphemes(input, true) {
        let is_hyphen = matches!(grapheme, "-" | "‐" | "‑");
        // An apostrophe opens a new segment only after a single-letter prefix
        // (O'Brien), never inside a contraction ("don't") or a possessive.
        let is_boundary_apostrophe =
            matches!(grapheme, "'" | "’") && letters_in_segment == 1;

        if is_hyphen || is_boundary_apostrophe {
            out.push_str(&mapper.titlecase_segment_to_string(&segment, &id, options));
            out.push_str(grapheme);
            segment.clear();
            letters_in_segment = 0;
            continue;
        }

        segment.push_str(grapheme);
        if grapheme.chars().any(char::is_alphabetic) {
            letters_in_segment += 1;
        }
    }
    out.push_str(&mapper.titlecase_segment_to_string(&segment, &id, options));
    out
}

/// Uppercases only the first grapheme, leaving the remainder untouched.
///
/// Unlike [`titlecase_word_locale`], this preserves internal casing, so it can
/// force a sentence-initial capital onto an already-cased form such as
/// `van der Waals` without flattening the rest to lowercase.
pub fn uppercase_first_grapheme(input: &str, locale: &str) -> String {
    let mut graphemes = UnicodeSegmentation::graphemes(input, true);
    match graphemes.next() {
        Some(first) => {
            let mut out = CaseMapper::new()
                .uppercase_to_string(first, &locale_id(locale))
                .into_owned();
            out.push_str(graphemes.as_str());
            out
        }
        None => String::new(),
    }
}

pub fn primary_language(locale: &str) -> &str {
    locale.split(['-', '_']).next().unwrap_or(locale)
}

/// Parses the locale's primary language into a [`LanguageIdentifier`] for ICU;
/// an unparseable locale falls back to the language-neutral root.
fn locale_id(locale: &str) -> LanguageIdentifier {
    primary_language(locale)
        .parse()
        .unwrap_or(LanguageIdentifier::UNKNOWN)
}
