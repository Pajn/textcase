use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "fr",
        stop_words: &[
            "à", "au", "aux", "de", "des", "du", "et", "la", "le", "les", "ou",
        ],
        lowercase_particles: &["de", "des", "du", "la", "le", "les", "d’", "l’"],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
