use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &["av", "den", "det", "en", "i", "och", "på", "som", "till"],
        numeric_abbreviations: &["nr", "vol", "fig"],
        trailing_abbreviations: &["osv", "etc"],
        ..LanguageProfile::neutral()
    }
}
