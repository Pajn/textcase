use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "fi",
        stop_words: &["ja", "mutta", "sekä", "tai", "vaan"],
        lowercase_particles: &[],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
