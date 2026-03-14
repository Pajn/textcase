use std::{fs, path::PathBuf};

use textcase::{
    lexicon::LoadedFstPlugin,
    plugin::{PluginSchema, inspect_plugin, inspect_plugin_metadata},
};

pub fn run(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let inspection = if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        let schema: PluginSchema = serde_json::from_slice(&fs::read(&path)?)?;
        inspect_plugin(&schema)
    } else {
        let plugin = LoadedFstPlugin::from_path(&path)?;
        inspect_plugin_metadata(&plugin.metadata, plugin.entry_count())
    };

    Ok(format!(
        "name: {}
kind: {}
entries: {}
locales: {}
sources: {}
license: {}
checksum: {}",
        inspection.name,
        inspection.kind,
        inspection.entry_count,
        inspection.locales.join(", "),
        inspection.sources.join(", "),
        inspection.license,
        inspection.checksum,
    ))
}
