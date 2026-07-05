use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "pt",
        stop_words: &[
            "a", "as", "da", "das", "de", "do", "dos", "e", "em", "na", "no", "para", "por",
        ],
        lowercase_particles: &["da", "das", "de", "do", "dos"],
        ..LanguageProfile::neutral()
    }
}
