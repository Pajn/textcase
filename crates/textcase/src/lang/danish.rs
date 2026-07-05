use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "da",
        stop_words: &["af", "den", "det", "en", "i", "og", "på", "til"],
        lowercase_particles: &[],
        ..LanguageProfile::neutral()
    }
}
