use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &["ar", "bei", "bet", "ir", "į", "iš", "su"],
        title_abbreviations: &["dr", "prof", "gerb"],
        numeric_abbreviations: &["nr", "vol"],
        trailing_abbreviations: &["pvz", "kt", "etc"],
        ..LanguageProfile::neutral()
    }
}
