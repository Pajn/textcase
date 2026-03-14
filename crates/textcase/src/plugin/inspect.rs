use crate::plugin::{PluginMetadata, PluginSchema};

#[derive(Clone, Debug)]
pub struct PluginInspection {
    pub name: String,
    pub kind: String,
    pub locales: Vec<String>,
    pub sources: Vec<String>,
    pub license: String,
    pub checksum: String,
}

impl From<&PluginSchema> for PluginInspection {
    fn from(schema: &PluginSchema) -> Self {
        from_metadata(&schema.metadata)
    }
}

pub fn inspect_plugin(metadata: &PluginMetadata) -> PluginInspection {
    from_metadata(metadata)
}

fn from_metadata(metadata: &PluginMetadata) -> PluginInspection {
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
    }
}
