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
    for grapheme in UnicodeSegmentation::graphemes(input, true) {
        if matches!(grapheme, "-" | "‐" | "‑" | "'" | "’") {
            out.push_str(grapheme);
            capitalize_next = true;
            continue;
        }

        if capitalize_next {
            out.push_str(&uppercase_locale(grapheme, locale));
            capitalize_next = false;
        } else {
            out.push_str(&lowercase_locale(grapheme, locale));
        }
    }
    out
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

fn primary_language(locale: &str) -> &str {
    locale.split(['-', '_']).next().unwrap_or(locale)
}
