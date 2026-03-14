use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OutputManifest {
    pub input: String,
    pub output: String,
    pub format: String,
    pub checksum: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchedSourceManifest {
    pub source: String,
    pub source_url: String,
    pub version: String,
    pub sample: bool,
}

pub fn write_source_manifest(
    raw_path: impl AsRef<Path>,
    manifest: &FetchedSourceManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = source_manifest_path(raw_path);
    fs::write(path, serde_json::to_vec_pretty(manifest)?)?;
    Ok(())
}

pub fn read_source_manifest(
    raw_path: impl AsRef<Path>,
) -> Result<Option<FetchedSourceManifest>, Box<dyn std::error::Error>> {
    let path = source_manifest_path(raw_path);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

pub fn source_manifest_path(raw_path: impl AsRef<Path>) -> PathBuf {
    raw_path.as_ref().with_extension("source.json")
}
