use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &["av", "de", "den", "det", "en", "i", "og", "på", "til"],
        title_abbreviations: &["dr", "prof", "hr", "fru", "st"],
        numeric_abbreviations: &["nr", "vol", "fig"],
        trailing_abbreviations: &["osv", "mv", "etc"],
        ..LanguageProfile::neutral()
    }
}
