use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "tr",
        stop_words: &["ama", "da", "de", "ile", "ve", "veya"],
        lowercase_particles: &[],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
