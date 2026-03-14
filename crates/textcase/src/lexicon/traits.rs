use serde::{Deserialize, Serialize};

/// A casing candidate returned by ranked lexicon lookups.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Candidate {
    /// The suggested canonical form.
    pub value: String,
    /// Relative confidence score; higher values win.
    pub score: f32,
}

/// Lookup interface used by the conversion pipeline to restore canonical forms.
pub trait LexiconProvider {
    /// Returns the canonical form for a single token when one is known.
    fn canonical_form(&self, locale: &str, token: &str) -> Option<String>;
    /// Returns the canonical form for a multiword phrase when one is known.
    fn canonical_phrase(&self, locale: &str, phrase: &str) -> Option<String>;
    /// Returns `true` when the provider contains the given token.
    fn contains_word(&self, locale: &str, token: &str) -> bool;
    /// Returns ranked casing candidates for the given token.
    fn ranked_candidates(&self, locale: &str, token: &str) -> Option<Vec<Candidate>>;
}
