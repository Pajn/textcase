pub fn is_sentence_terminal(text: &str) -> bool {
    matches!(text, "." | "!" | "?" | "…")
}

pub fn is_subtitle_separator(text: &str) -> bool {
    matches!(text, ":" | "-" | "–" | "—")
}
