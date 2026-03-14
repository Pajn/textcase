use super::{ambiguity::AMBIGUOUS_LOWERCASE, contexts};

pub const NOUN_SUFFIXES: &[&str] = &[
    "ung", "heit", "keit", "schaft", "tion", "ismus", "ität", "ling", "tum", "nis",
];

pub fn looks_like_noun_context(
    current: &str,
    previous: Option<&str>,
    previous2: Option<&str>,
) -> bool {
    if AMBIGUOUS_LOWERCASE.contains(&current) {
        return false;
    }

    if current.len() <= 2 {
        return false;
    }

    if NOUN_SUFFIXES.iter().any(|suffix| current.ends_with(suffix)) {
        return true;
    }

    if let Some(prev) = previous {
        if contexts::ARTICLES.contains(&prev) {
            return true;
        }
    }

    if let (Some(prev), Some(prev2)) = (previous, previous2) {
        if contexts::PREPOSITIONS.contains(&prev2) && contexts::ARTICLES.contains(&prev) {
            return true;
        }
    }

    false
}
