use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "fr",
        stop_words: &[
            "à", "au", "aux", "de", "des", "du", "et", "la", "le", "les", "ou",
        ],
        lowercase_particles: &["de", "des", "du", "la", "le", "les", "d’", "l’"],
        title_abbreviations: &["dr", "prof", "st", "ste", "mme", "mlle"],
        elision_prefixes: &[
            "l", "d", "j", "n", "m", "t", "s", "c", "qu", "jusqu", "lorsqu", "puisqu", "quoiqu",
        ],
        ..LanguageProfile::neutral()
    }
}
