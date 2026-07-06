pub fn builtin_canonical_form(token: &str) -> Option<&'static str> {
    match token {
        "github" => Some("GitHub"),
        "latex" => Some("LaTeX"),
        "icu4x" => Some("ICU4X"),
        "iphone" => Some("iPhone"),
        "ipad" => Some("iPad"),
        "rust" => Some("Rust"),
        _ => None,
    }
}

/// Whether a builtin form's lowercase spelling is also an ordinary word
/// ("rust" the metal oxide, "latex" the material), so restoring it needs a
/// casing signal from the input rather than firing on every occurrence.
pub fn builtin_form_is_ambiguous(token: &str) -> bool {
    matches!(token, "rust" | "latex")
}

pub fn builtin_canonical_phrase(phrase: &str) -> Option<&'static str> {
    match phrase {
        "new york" => Some("New York"),
        "san francisco" => Some("San Francisco"),
        "van der waals" => Some("van der Waals"),
        _ => None,
    }
}
