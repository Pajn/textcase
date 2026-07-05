use unicode_segmentation::UnicodeSegmentation;

pub fn lowercase_locale(input: &str, locale: &str) -> String {
    let language = primary_language(locale);
    match language {
        "tr" | "az" => input
            .chars()
            .flat_map(|ch| match ch {
                'I' => "ı".chars().collect::<Vec<_>>(),
                'İ' => "i".chars().collect::<Vec<_>>(),
                _ => ch.to_lowercase().collect(),
            })
            .collect(),
        _ => input.to_lowercase(),
    }
}

pub fn capitalize_word_locale(input: &str, locale: &str) -> String {
    titlecase_word_locale(input, locale)
}

pub fn titlecase_word_locale(input: &str, locale: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;
    // Number of letters emitted in the current segment since the last boundary,
    // used to decide whether an apostrophe introduces a new capital.
    let mut segment_len = 0usize;
    for grapheme in UnicodeSegmentation::graphemes(input, true) {
        if matches!(grapheme, "-" | "‐" | "‑") {
            out.push_str(grapheme);
            capitalize_next = true;
            segment_len = 0;
            continue;
        }

        if matches!(grapheme, "'" | "’") {
            out.push_str(grapheme);
            // Capitalize after an apostrophe only for a single-letter prefix
            // (O'Brien, D'Angelo), never for contractions ("don't") or
            // possessives ("James'").
            capitalize_next = segment_len == 1;
            segment_len = 0;
            continue;
        }

        if capitalize_next {
            out.push_str(&uppercase_locale(grapheme, locale));
            capitalize_next = false;
        } else {
            out.push_str(&lowercase_locale(grapheme, locale));
        }
        segment_len += 1;
    }
    out
}

/// Uppercases only the first grapheme, leaving the remainder untouched.
///
/// Unlike [`capitalize_word_locale`], this preserves internal casing, so it can
/// force a sentence-initial capital onto an already-cased form such as
/// `van der Waals` without flattening the rest to lowercase.
pub fn uppercase_first_grapheme(input: &str, locale: &str) -> String {
    let mut graphemes = UnicodeSegmentation::graphemes(input, true);
    match graphemes.next() {
        Some(first) => {
            let mut out = uppercase_locale(first, locale);
            out.push_str(graphemes.as_str());
            out
        }
        None => String::new(),
    }
}

fn uppercase_locale(input: &str, locale: &str) -> String {
    let language = primary_language(locale);
    match language {
        "tr" | "az" => input
            .chars()
            .flat_map(|ch| match ch {
                'i' => "İ".chars().collect::<Vec<_>>(),
                'ı' => "I".chars().collect::<Vec<_>>(),
                _ => ch.to_uppercase().collect(),
            })
            .collect(),
        _ => input.to_uppercase(),
    }
}

pub fn primary_language(locale: &str) -> &str {
    locale.split(['-', '_']).next().unwrap_or(locale)
}
