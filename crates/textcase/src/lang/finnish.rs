use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        stop_words: &["ja", "mutta", "sekä", "tai", "vaan"],
        title_abbreviations: &["prof", "tri", "hra", "rva"],
        numeric_abbreviations: &["nro", "vol"],
        trailing_abbreviations: &["jne", "ym", "esim", "etc"],
        ..LanguageProfile::neutral()
    }
}
