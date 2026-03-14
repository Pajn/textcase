use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::lexicon::Candidate;

use super::metadata::{LicenseMetadata, SourceMetadata};

/// Semantic version for the JSON plugin schema.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SchemaVersion {
    /// Major schema version.
    pub major: u16,
    /// Minor schema version.
    pub minor: u16,
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}

/// Supported plugin payload kinds.
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
    /// Returns the stable kebab-case name used in serialized plugin files.
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

/// Shared metadata stored in every plugin container.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PluginMetadata {
    /// JSON schema version.
    pub schema: SchemaVersion,
    /// Human-readable plugin name.
    pub name: String,
    /// Payload kind carried by this plugin.
    pub kind: PluginKind,
    /// Supported locales.
    pub locales: Vec<String>,
    /// License summary for the derived payload.
    pub license: LicenseMetadata,
    /// Provenance for the upstream sources.
    pub sources: Vec<SourceMetadata>,
    /// RFC 3339 generation timestamp.
    pub generated_at: String,
    /// Payload checksum.
    pub checksum: String,
}

/// Payload values stored in a JSON plugin.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "payload_kind", content = "payload", rename_all = "kebab-case")]
pub enum PluginPayload {
    WordSet(Vec<String>),
    CanonicalMap(BTreeMap<String, String>),
    MultiwordMap(BTreeMap<String, String>),
    RankedCandidates(BTreeMap<String, Vec<Candidate>>),
    ProtectedForms(BTreeMap<String, String>),
}

/// Full JSON plugin document combining metadata and payload.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PluginSchema {
    #[serde(flatten)]
    pub metadata: PluginMetadata,
    #[serde(flatten)]
    pub payload: PluginPayload,
}
