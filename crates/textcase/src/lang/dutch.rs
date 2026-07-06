use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "nl",
        stop_words: &[
            "aan", "de", "den", "der", "en", "het", "in", "of", "op", "te", "van",
        ],
        lowercase_particles: &["de", "den", "der", "het", "te", "van"],
        title_abbreviations: &["dhr", "mevr", "dr", "prof", "st"],
        numeric_abbreviations: &["nr", "vol", "fig"],
        trailing_abbreviations: &["enz", "etc"],
        ..LanguageProfile::neutral()
    }
}
