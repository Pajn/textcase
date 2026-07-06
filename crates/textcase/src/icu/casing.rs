use icu_casemap::{CaseMapper, TitlecaseMapper, options::TitlecaseOptions};
use icu_locale_core::LanguageIdentifier;
use unicode_segmentation::UnicodeSegmentation;

pub fn lowercase_locale(input: &str, locale: &str) -> String {
    CaseMapper::new()
        .lowercase_to_string(input, &locale_id(locale))
        .into_owned()
}

/// Capitalizes a word for sentence-start position: the first cased character is
/// upper-cased and the rest lower-cased, treating the whole word as one segment.
///
/// Unlike [`titlecase_word_locale`] this does *not* split on hyphens or
/// apostrophes, so `re-enter` becomes `Re-enter` and `jean-paul` becomes
/// `Jean-paul` — sentence case capitalizes only the first letter of the
/// sentence. Locale rules still apply (Dutch `ijssel` → `IJssel`).
pub fn capitalize_word_locale(input: &str, locale: &str) -> String {
    TitlecaseMapper::new()
        .titlecase_segment_to_string(input, &locale_id(locale), TitlecaseOptions::default())
        .into_owned()
}

/// Title-cases a single word, capitalizing the first letter of each segment.
///
/// Segments are split on hyphens and on an apostrophe that follows a
/// single-letter prefix (`O'Brien`) or an elided particle (`d'affaires`); an
/// apostrophe inside a contraction (`don't`) stays within its segment. The
/// contraction tails and elision prefixes come from the language profile. An
/// elided particle stays lowercase (`d'Affaires`); other segments are
/// title-cased through ICU, so locale rules apply — e.g. Dutch `ijssel`
/// becomes `IJssel`.
pub fn titlecase_word_locale(
    input: &str,
    locale: &str,
    contraction_tails: &[&str],
    elision_prefixes: &[&str],
) -> String {
    let mapper = TitlecaseMapper::new();
    let id = locale_id(locale);
    let options = TitlecaseOptions::default();

    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(input, true).collect();
    let mut out = String::with_capacity(input.len());
    let mut segment = String::new();
    let mut letters_in_segment = 0usize;

    for (index, &grapheme) in graphemes.iter().enumerate() {
        let is_hyphen = matches!(grapheme, "-" | "‐" | "‑");
        let is_apostrophe = matches!(grapheme, "'" | "’");
        let is_elision =
            is_apostrophe && elision_prefixes.contains(&segment.to_lowercase().as_str());
        // An apostrophe opens a new segment after an elided particle
        // ("l'homme", "qu'elle") or a single-letter prefix (O'Brien), but not
        // inside a contraction ("don't", "I'm", "y'all") or a possessive.
        let is_boundary_apostrophe = is_apostrophe
            && (is_elision
                || (letters_in_segment == 1
                    && !is_contraction_suffix(&graphemes[index + 1..], contraction_tails)));

        if is_hyphen || is_boundary_apostrophe {
            if is_elision {
                out.push_str(&CaseMapper::new().lowercase_to_string(&segment, &id));
            } else {
                out.push_str(&mapper.titlecase_segment_to_string(&segment, &id, options));
            }
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

/// Whether the graphemes following an apostrophe form a known contraction
/// tail. A single-letter prefix plus such a tail is a contraction ("I'm",
/// "y'all", "o'clock") that must stay one segment, rather than a name particle
/// like "O'Brien" that opens a new segment.
fn is_contraction_suffix(rest: &[&str], contraction_tails: &[&str]) -> bool {
    let mut tail = String::new();
    for &grapheme in rest {
        if matches!(grapheme, "-" | "‐" | "‑" | "'" | "’") {
            break;
        }
        tail.push_str(grapheme);
    }
    contraction_tails.contains(&tail.to_lowercase().as_str())
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
