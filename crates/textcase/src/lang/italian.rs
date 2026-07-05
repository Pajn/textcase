use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "it",
        stop_words: &[
            "a", "da", "de", "del", "della", "di", "e", "il", "in", "la", "le", "lo",
        ],
        lowercase_particles: &["da", "de", "del", "della", "di"],
        title_abbreviations: &["sig", "dott", "prof", "dr", "st"],
        trailing_abbreviations: &["ecc", "etc"],
        ..LanguageProfile::neutral()
    }
}
