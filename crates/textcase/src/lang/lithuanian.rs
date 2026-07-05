use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "lt",
        stop_words: &["ar", "bei", "bet", "ir", "į", "iš", "su"],
        lowercase_particles: &[],
        ..LanguageProfile::neutral()
    }
}
