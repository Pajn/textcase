use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "nl",
        stop_words: &[
            "aan", "de", "den", "der", "en", "het", "in", "of", "op", "te", "van",
        ],
        lowercase_particles: &["de", "den", "der", "het", "te", "van"],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
