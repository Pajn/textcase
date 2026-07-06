use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "az",
        stop_words: &["amma", "da", "də", "ilə", "və", "ya"],
        lowercase_particles: &[],
        ..LanguageProfile::neutral()
    }
}
