use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::plugin::{
    LicenseMetadata, PluginKind, PluginMetadata, PluginPayload, PluginSchema, SchemaVersion,
    SourceMetadata,
};
use crate::util::checksum_hex;

use super::Candidate;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PreparedKind {
    WordSet,
    CanonicalMap,
    MultiwordMap,
    RankedCandidates,
    ProtectedForms,
}

impl PreparedKind {
    pub fn to_plugin_kind(self) -> PluginKind {
        match self {
            PreparedKind::WordSet => PluginKind::WordSet,
            PreparedKind::CanonicalMap => PluginKind::CanonicalMap,
            PreparedKind::MultiwordMap => PluginKind::MultiwordMap,
            PreparedKind::RankedCandidates => PluginKind::RankedCandidates,
            PreparedKind::ProtectedForms => PluginKind::ProtectedForms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "payload_kind", content = "payload", rename_all = "kebab-case")]
pub enum PreparedPayload {
    WordSet(Vec<String>),
    CanonicalMap(BTreeMap<String, String>),
    MultiwordMap(BTreeMap<String, String>),
    RankedCandidates(BTreeMap<String, Vec<Candidate>>),
    ProtectedForms(BTreeMap<String, String>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PreparedLexicon {
    pub name: String,
    pub kind: PreparedKind,
    pub locale: String,
    pub license: LicenseMetadata,
    pub sources: Vec<SourceMetadata>,
    pub generated_at: String,
    pub payload: PreparedPayload,
}

impl PreparedLexicon {
    pub fn to_plugin_schema(&self) -> PluginSchema {
        let payload = match &self.payload {
            PreparedPayload::WordSet(values) => PluginPayload::WordSet(values.clone()),
            PreparedPayload::CanonicalMap(values) => PluginPayload::CanonicalMap(values.clone()),
            PreparedPayload::MultiwordMap(values) => PluginPayload::MultiwordMap(values.clone()),
            PreparedPayload::RankedCandidates(values) => {
                PluginPayload::RankedCandidates(values.clone())
            }
            PreparedPayload::ProtectedForms(values) => {
                PluginPayload::ProtectedForms(values.clone())
            }
        };

        let payload_bytes =
            serde_json::to_vec(&payload).expect("payload serialization should succeed");
        PluginSchema {
            metadata: PluginMetadata {
                schema: SchemaVersion::default(),
                name: self.name.clone(),
                kind: self.kind.to_plugin_kind(),
                locales: vec![self.locale.clone()],
                license: self.license.clone(),
                sources: self.sources.clone(),
                generated_at: self.generated_at.clone(),
                checksum: checksum_hex(&payload_bytes),
            },
            payload,
        }
    }
}
