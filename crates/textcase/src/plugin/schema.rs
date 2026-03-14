use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::lexicon::Candidate;

use super::metadata::{LicenseMetadata, SourceMetadata};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SchemaVersion {
    pub major: u16,
    pub minor: u16,
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    WordSet,
    CanonicalMap,
    MultiwordMap,
    RankedCandidates,
    ProtectedForms,
}

impl PluginKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PluginKind::WordSet => "word-set",
            PluginKind::CanonicalMap => "canonical-map",
            PluginKind::MultiwordMap => "multiword-map",
            PluginKind::RankedCandidates => "ranked-candidates",
            PluginKind::ProtectedForms => "protected-forms",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PluginMetadata {
    pub schema: SchemaVersion,
    pub name: String,
    pub kind: PluginKind,
    pub locales: Vec<String>,
    pub license: LicenseMetadata,
    pub sources: Vec<SourceMetadata>,
    pub generated_at: String,
    pub checksum: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "payload_kind", content = "payload", rename_all = "kebab-case")]
pub enum PluginPayload {
    WordSet(Vec<String>),
    CanonicalMap(BTreeMap<String, String>),
    MultiwordMap(BTreeMap<String, String>),
    RankedCandidates(BTreeMap<String, Vec<Candidate>>),
    ProtectedForms(BTreeMap<String, String>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PluginSchema {
    #[serde(flatten)]
    pub metadata: PluginMetadata,
    #[serde(flatten)]
    pub payload: PluginPayload,
}
