use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "lt",
        stop_words: &["ar", "bei", "bet", "ir", "į", "iš", "su"],
        lowercase_particles: &[],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
