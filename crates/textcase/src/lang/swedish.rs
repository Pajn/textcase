use super::LanguageProfile;

pub fn profile() -> LanguageProfile {
    LanguageProfile {
        locale: "sv",
        stop_words: &["av", "den", "det", "en", "i", "och", "på", "som", "till"],
        lowercase_particles: &[],
        ..LanguageProfile::neutral()
    }
}
