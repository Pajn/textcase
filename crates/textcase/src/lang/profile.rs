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
    /// A language-neutral profile for locales without dedicated data. It
    /// assumes nothing about the language beyond a few Latin abbreviations,
    /// rather than borrowing English stop words and particles.
    pub const fn neutral() -> Self {
        Self {
            locale: "und",
            stop_words: &[],
            lowercase_particles: &[],
            noun_articles: &[],
            noun_prepositions: &[],
            noun_suffixes: &[],
            ambiguous_lowercase: &[],
        }
    }

    pub fn keeps_lowercase_in_title(self, token: &str) -> bool {
        self.stop_words.contains(&token)
    }

    pub fn keeps_particle_lowercase(self, token: &str) -> bool {
        self.lowercase_particles.contains(&token)
    }
}
