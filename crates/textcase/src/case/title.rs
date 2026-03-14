use crate::lang::LanguageProfile;

pub fn should_keep_lowercase_in_title(
    profile: LanguageProfile,
    token: &str,
    is_edge_word: bool,
) -> bool {
    !is_edge_word
        && (profile.keeps_lowercase_in_title(token) || profile.keeps_particle_lowercase(token))
}
