pub fn should_capitalize_after_separator(capitalize_after_separator: bool, token: &str) -> bool {
    capitalize_after_separator && matches!(token, ":" | "-" | "–" | "—")
}
