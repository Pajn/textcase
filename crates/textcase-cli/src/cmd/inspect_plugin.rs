use std::{
    fs,
    path::{Path, PathBuf},
};

use textcase::plugin::{PluginMetadata, PluginSchema, inspect_plugin};

pub fn run(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let inspection = if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        let schema: PluginSchema = serde_json::from_slice(&fs::read(&path)?)?;
        textcase::plugin::inspect_plugin(&schema.metadata)
    } else {
        let metadata = read_fst_metadata(&path)?;
        inspect_plugin(&metadata)
    };

    Ok(format!(
        "name: {}
kind: {}
locales: {}
sources: {}
license: {}
checksum: {}",
        inspection.name,
        inspection.kind,
        inspection.locales.join(", "),
        inspection.sources.join(", "),
        inspection.license,
        inspection.checksum,
    ))
}

fn read_fst_metadata(path: &Path) -> Result<PluginMetadata, Box<dyn std::error::Error>> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid plugin filename")?;
    let sidecar = path.with_file_name(format!("{filename}.meta.json"));
    let sidecar: textcase::lexicon::FstSidecar = serde_json::from_slice(&fs::read(sidecar)?)?;
    Ok(sidecar.metadata)
}
