use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &[
            "a", "da", "de", "del", "della", "di", "e", "il", "in", "la", "le", "lo",
        ],
        lowercase_particles: &["da", "de", "del", "della", "di"],
        title_abbreviations: &["sig", "dott", "prof", "dr", "st"],
        trailing_abbreviations: &["ecc", "etc"],
        elision_prefixes: &["l", "d", "un", "all", "dell", "nell", "sull", "sant"],
        ..LanguageProfile::neutral()
    }
}
