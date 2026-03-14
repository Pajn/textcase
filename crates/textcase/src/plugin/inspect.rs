use crate::plugin::{PluginMetadata, PluginSchema};

#[derive(Clone, Debug)]
pub struct PluginInspection {
    pub name: String,
    pub kind: String,
    pub locales: Vec<String>,
    pub sources: Vec<String>,
    pub license: String,
    pub checksum: String,
    pub entry_count: usize,
}

impl From<&PluginSchema> for PluginInspection {
    fn from(schema: &PluginSchema) -> Self {
        from_metadata(&schema.metadata, payload_entry_count(&schema.payload))
    }
}

pub fn inspect_plugin(schema: &PluginSchema) -> PluginInspection {
    schema.into()
}

pub fn inspect_plugin_metadata(metadata: &PluginMetadata, entry_count: usize) -> PluginInspection {
    from_metadata(metadata, entry_count)
}

fn from_metadata(metadata: &PluginMetadata, entry_count: usize) -> PluginInspection {
    PluginInspection {
        name: metadata.name.clone(),
        kind: metadata.kind.as_str().to_string(),
        locales: metadata.locales.clone(),
        sources: metadata
            .sources
            .iter()
            .map(|source| source.id.clone())
            .collect(),
        license: format!("{}: {}", metadata.license.name, metadata.license.summary),
        checksum: metadata.checksum.clone(),
        entry_count,
    }
}

fn payload_entry_count(payload: &crate::plugin::PluginPayload) -> usize {
    match payload {
        crate::plugin::PluginPayload::WordSet(values) => values.len(),
        crate::plugin::PluginPayload::CanonicalMap(values) => values.len(),
        crate::plugin::PluginPayload::MultiwordMap(values) => values.len(),
        crate::plugin::PluginPayload::RankedCandidates(values) => values.len(),
        crate::plugin::PluginPayload::ProtectedForms(values) => values.len(),
    }
}
