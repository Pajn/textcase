use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "no",
        stop_words: &["av", "de", "den", "det", "en", "i", "og", "på", "til"],
        lowercase_particles: &[],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
