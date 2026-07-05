use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "tr",
        stop_words: &["ama", "da", "de", "ile", "ve", "veya"],
        title_abbreviations: &["dr", "prof", "av", "doç"],
        numeric_abbreviations: &["no", "vol", "fig"],
        // Unlike English "vs.", Turkish "vs." (vesaire) trails like "etc.".
        trailing_abbreviations: &["vb", "vs", "vd", "etc"],
        ..LanguageProfile::neutral()
    }
}
