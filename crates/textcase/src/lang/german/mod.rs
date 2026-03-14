mod ambiguity;
mod contexts;
mod heuristics;
mod rules;

use super::LanguageProfile;

pub use rules::recase_token;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "de",
        stop_words: &[
            "am", "an", "auf", "der", "die", "das", "ein", "eine", "für", "im", "in", "mit",
            "oder", "und", "von", "zu",
        ],
        lowercase_particles: &[
            "am", "an", "auf", "der", "die", "das", "ein", "eine", "im", "in", "mit", "von", "zu",
        ],
        noun_articles: contexts::ARTICLES,
        noun_prepositions: contexts::PREPOSITIONS,
        noun_suffixes: heuristics::NOUN_SUFFIXES,
        ambiguous_lowercase: ambiguity::AMBIGUOUS_LOWERCASE,
    }
}
