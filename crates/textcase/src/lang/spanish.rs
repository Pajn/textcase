use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "es",
        stop_words: &[
            "a", "al", "con", "de", "del", "el", "en", "la", "las", "los", "o", "por", "para", "y",
        ],
        lowercase_particles: &["de", "del", "la", "las", "los"],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
