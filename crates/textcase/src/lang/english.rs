use super::LanguageProfile;

/// The pronoun "I" and its contractions are capitalized wherever they appear;
/// this is a hard rule of English orthography, not a lexicon matter.
pub fn always_capitalized(lower: &str) -> bool {
    let normalized = lower.replace('’', "'");
    matches!(normalized.as_str(), "i" | "i'm" | "i'll" | "i've" | "i'd")
}

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "en",
        stop_words: &[
            "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "per",
            "the", "to", "vs", "via",
        ],
        lowercase_particles: &["de", "du", "van", "von"],
        title_abbreviations: &[
            "mr", "mrs", "ms", "dr", "prof", "st", "sr", "jr", "vs", "feat", "ft", "capt", "sgt",
            "col", "gen", "gov",
        ],
        numeric_abbreviations: &["no", "vol", "fig", "approx"],
        trailing_abbreviations: &["etc", "co", "inc", "ltd", "dept"],
        contraction_tails: &["m", "ll", "ve", "re", "d", "s", "t", "all", "clock", "em"],
        ..LanguageProfile::neutral()
    }
}
