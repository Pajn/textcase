use crate::tokenize::AbbreviationKind;

#[derive(Clone, Copy, Debug)]
pub struct LanguageProfile {
    pub stop_words: &'static [&'static str],
    pub lowercase_particles: &'static [&'static str],
    /// Abbreviations followed by a name or a continuing phrase ("Dr. Smith");
    /// a period after one never ends the sentence.
    pub title_abbreviations: &'static [&'static str],
    /// Abbreviations only used directly before a number ("No. 5").
    pub numeric_abbreviations: &'static [&'static str],
    /// Abbreviations that can close a phrase or a whole sentence ("etc.").
    pub trailing_abbreviations: &'static [&'static str],
    /// Apostrophe contraction tails that keep a word in one titlecase segment
    /// ("don't", "o'clock") instead of opening a name segment ("O'Brien").
    pub contraction_tails: &'static [&'static str],
    /// Elided particles before an apostrophe ("l'", "d'", "qu'"). In a title
    /// the particle stays lowercase while the following segment is
    /// capitalized: "d'affaires" becomes "d'Affaires".
    pub elision_prefixes: &'static [&'static str],
}

impl LanguageProfile {
    /// A language-neutral profile for locales without dedicated data. It
    /// assumes nothing about the language beyond a few Latin abbreviations
    /// shared across European typography, rather than borrowing English stop
    /// words and particles.
    pub const fn neutral() -> Self {
        Self {
            stop_words: &[],
            lowercase_particles: &[],
            title_abbreviations: &["dr", "prof", "st"],
            numeric_abbreviations: &["no", "vol", "fig"],
            trailing_abbreviations: &["etc"],
            contraction_tails: &[],
            elision_prefixes: &[],
        }
    }

    pub fn keeps_lowercase_in_title(self, token: &str) -> bool {
        self.stop_words.contains(&token)
    }

    pub fn keeps_particle_lowercase(self, token: &str) -> bool {
        self.lowercase_particles.contains(&token)
    }

    /// Classifies a lowercased word as an abbreviation of this language, so
    /// callers can decide from context whether a period after it ends the
    /// sentence.
    pub fn abbreviation_kind(self, word: &str) -> Option<AbbreviationKind> {
        if self.title_abbreviations.contains(&word) {
            Some(AbbreviationKind::Title)
        } else if self.numeric_abbreviations.contains(&word) {
            Some(AbbreviationKind::Numeric)
        } else if self.trailing_abbreviations.contains(&word) {
            Some(AbbreviationKind::Trailing)
        } else {
            None
        }
    }
}
