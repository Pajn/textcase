use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use fst::Streamer;

use crate::{
    Result,
    lexicon::json::load_json_plugin,
    plugin::{PluginKind, PluginPayload},
};

use super::{
    Candidate, LexiconProvider,
    fst::{FstPayload, LoadedFstPlugin},
};

/// A mergeable lexicon container backed by JSON and/or FST plugins.
#[derive(Clone, Debug, Default)]
pub struct PluginSet {
    word_sets: BTreeMap<String, BTreeSet<String>>,
    canonical_maps: BTreeMap<String, BTreeMap<String, String>>,
    phrase_maps: BTreeMap<String, BTreeMap<String, String>>,
    ranked_candidates: BTreeMap<String, BTreeMap<String, Vec<Candidate>>>,
    protected_forms: BTreeMap<String, BTreeMap<String, String>>,
}

impl PluginSet {
    /// Builds a plugin set from a JSON plugin payload.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        let schema = load_json_plugin(bytes)?;
        let mut set = Self::default();
        set.ingest_payload(schema.metadata.locales, schema.payload);
        Ok(set)
    }

    /// Builds a plugin set from an on-disk FST plugin.
    pub fn from_fst_path(path: impl AsRef<Path>) -> Result<Self> {
        let plugin = LoadedFstPlugin::from_path(path)?;
        let mut set = Self::default();
        let locale = plugin
            .metadata
            .locales
            .first()
            .cloned()
            .unwrap_or_else(|| "und".to_string());
        match plugin.payload {
            FstPayload::Set(set_payload) => {
                let entries = set_payload.stream().into_strs()?;
                let word_set = set.word_sets.entry(locale).or_default();
                word_set.extend(entries);
            }
            FstPayload::Map(map_payload) => {
                let mut stream = map_payload.stream();
                while let Some((key, value)) = stream.next() {
                    let key = String::from_utf8_lossy(key).to_string();
                    match plugin.metadata.kind {
                        PluginKind::CanonicalMap => {
                            if let Some(mapped) = plugin.values.get(value as usize) {
                                set.canonical_maps
                                    .entry(locale.clone())
                                    .or_default()
                                    .insert(key, mapped.clone());
                            }
                        }
                        PluginKind::MultiwordMap => {
                            if let Some(mapped) = plugin.values.get(value as usize) {
                                set.phrase_maps
                                    .entry(locale.clone())
                                    .or_default()
                                    .insert(key, mapped.clone());
                            }
                        }
                        PluginKind::ProtectedForms => {
                            if let Some(mapped) = plugin.values.get(value as usize) {
                                set.protected_forms
                                    .entry(locale.clone())
                                    .or_default()
                                    .insert(key, mapped.clone());
                            }
                        }
                        PluginKind::RankedCandidates => {
                            if let Some(mapped) = plugin.candidate_values.get(value as usize) {
                                set.ranked_candidates
                                    .entry(locale.clone())
                                    .or_default()
                                    .insert(key, mapped.clone());
                            }
                        }
                        PluginKind::WordSet => {}
                    }
                }
            }
        }
        Ok(set)
    }

    /// Merges two plugin sets, giving entries from `other` precedence on key collisions.
    pub fn merge(mut self, other: Self) -> Self {
        merge_map_set(&mut self.word_sets, other.word_sets);
        merge_nested_map(&mut self.canonical_maps, other.canonical_maps);
        merge_nested_map(&mut self.phrase_maps, other.phrase_maps);
        merge_nested_map(&mut self.protected_forms, other.protected_forms);
        merge_ranked(&mut self.ranked_candidates, other.ranked_candidates);
        self
    }

    fn ingest_payload(&mut self, locales: Vec<String>, payload: PluginPayload) {
        for locale in locales {
            match &payload {
                PluginPayload::WordSet(values) => {
                    self.word_sets
                        .entry(locale.clone())
                        .or_default()
                        .extend(values.iter().cloned());
                }
                PluginPayload::CanonicalMap(values) => {
                    self.canonical_maps
                        .entry(locale.clone())
                        .or_default()
                        .extend(values.clone());
                }
                PluginPayload::MultiwordMap(values) => {
                    self.phrase_maps
                        .entry(locale.clone())
                        .or_default()
                        .extend(values.clone());
                }
                PluginPayload::RankedCandidates(values) => {
                    self.ranked_candidates
                        .entry(locale.clone())
                        .or_default()
                        .extend(values.clone());
                }
                PluginPayload::ProtectedForms(values) => {
                    self.protected_forms
                        .entry(locale.clone())
                        .or_default()
                        .extend(values.clone());
                }
            }
        }
    }
}

impl LexiconProvider for PluginSet {
    fn canonical_form(&self, locale: &str, token: &str) -> Option<String> {
        let key = token.trim();
        self.protected_forms
            .get(locale)
            .and_then(|map| map.get(key).cloned())
            .or_else(|| {
                self.canonical_maps
                    .get(locale)
                    .and_then(|map| map.get(key).cloned())
            })
            .or_else(|| {
                locale.split('-').next().and_then(|language| {
                    self.protected_forms
                        .get(language)
                        .and_then(|map| map.get(key).cloned())
                })
            })
            .or_else(|| {
                locale.split('-').next().and_then(|language| {
                    self.canonical_maps
                        .get(language)
                        .and_then(|map| map.get(key).cloned())
                })
            })
    }

    fn canonical_phrase(&self, locale: &str, phrase: &str) -> Option<String> {
        self.phrase_maps
            .get(locale)
            .and_then(|map| map.get(phrase.trim()).cloned())
            .or_else(|| {
                locale.split('-').next().and_then(|language| {
                    self.phrase_maps
                        .get(language)
                        .and_then(|map| map.get(phrase.trim()).cloned())
                })
            })
    }

    fn contains_word(&self, locale: &str, token: &str) -> bool {
        self.word_sets
            .get(locale)
            .map(|set| set.contains(token))
            .unwrap_or(false)
    }

    fn ranked_candidates(&self, locale: &str, token: &str) -> Option<Vec<Candidate>> {
        self.ranked_candidates
            .get(locale)
            .and_then(|map| map.get(token).cloned())
            .or_else(|| {
                locale.split('-').next().and_then(|language| {
                    self.ranked_candidates
                        .get(language)
                        .and_then(|map| map.get(token).cloned())
                })
            })
    }
}

fn merge_map_set(
    target: &mut BTreeMap<String, BTreeSet<String>>,
    other: BTreeMap<String, BTreeSet<String>>,
) {
    for (locale, values) in other {
        target.entry(locale).or_default().extend(values);
    }
}

fn merge_nested_map(
    target: &mut BTreeMap<String, BTreeMap<String, String>>,
    other: BTreeMap<String, BTreeMap<String, String>>,
) {
    for (locale, values) in other {
        target.entry(locale).or_default().extend(values);
    }
}

fn merge_ranked(
    target: &mut BTreeMap<String, BTreeMap<String, Vec<Candidate>>>,
    other: BTreeMap<String, BTreeMap<String, Vec<Candidate>>>,
) {
    for (locale, values) in other {
        target.entry(locale).or_default().extend(values);
    }
}
