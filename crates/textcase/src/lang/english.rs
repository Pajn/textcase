use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "en",
        stop_words: &[
            "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "per",
            "the", "to", "vs", "via",
        ],
        lowercase_particles: &["de", "du", "van", "von"],
        noun_articles: &[],
        noun_prepositions: &[],
        noun_suffixes: &[],
        ambiguous_lowercase: &[],
    }
}
