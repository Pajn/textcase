use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Candidate {
    pub value: String,
    pub score: f32,
}

pub trait LexiconProvider {
    fn canonical_form(&self, locale: &str, token: &str) -> Option<String>;
    fn canonical_phrase(&self, locale: &str, phrase: &str) -> Option<String>;
    fn contains_word(&self, locale: &str, token: &str) -> bool;
    fn ranked_candidates(&self, locale: &str, token: &str) -> Option<Vec<Candidate>>;
}
