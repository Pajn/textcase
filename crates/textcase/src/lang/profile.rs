#[derive(Clone, Copy, Debug)]
pub struct LanguageProfile {
    pub locale: &'static str,
    pub stop_words: &'static [&'static str],
    pub lowercase_particles: &'static [&'static str],
    pub noun_articles: &'static [&'static str],
    pub noun_prepositions: &'static [&'static str],
    pub noun_suffixes: &'static [&'static str],
    pub ambiguous_lowercase: &'static [&'static str],
}

impl LanguageProfile {
    pub fn keeps_lowercase_in_title(self, token: &str) -> bool {
        self.stop_words.contains(&token)
    }

    pub fn keeps_particle_lowercase(self, token: &str) -> bool {
        self.lowercase_particles.contains(&token)
    }
}
