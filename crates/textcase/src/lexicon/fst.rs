use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use fst::{Map, MapBuilder, Set, SetBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    error::Error,
    plugin::{PluginKind, PluginMetadata},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FstSidecar {
    pub metadata: PluginMetadata,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub candidate_values: Vec<Vec<crate::lexicon::Candidate>>,
}

pub enum FstPayload {
    Set(Set<Vec<u8>>),
    Map(Map<Vec<u8>>),
}

pub struct LoadedFstPlugin {
    pub metadata: PluginMetadata,
    pub payload: FstPayload,
    pub values: Vec<String>,
    pub candidate_values: Vec<Vec<crate::lexicon::Candidate>>,
}

impl LoadedFstPlugin {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path)?;
        let sidecar_path = sidecar_path(path);
        if !sidecar_path.exists() {
            return Err(Error::MissingPluginMetadata(sidecar_path));
        }
        let sidecar: FstSidecar = serde_json::from_slice(&fs::read(&sidecar_path)?)?;
        let payload = match sidecar.metadata.kind {
            PluginKind::WordSet => FstPayload::Set(Set::new(bytes)?),
            _ => FstPayload::Map(Map::new(bytes)?),
        };
        Ok(Self {
            metadata: sidecar.metadata,
            payload,
            values: sidecar.values,
            candidate_values: sidecar.candidate_values,
        })
    }
}

pub fn write_set(path: impl AsRef<Path>, values: &[String], sidecar: &FstSidecar) -> Result<()> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut builder = SetBuilder::memory();
    for value in sorted {
        builder.insert(value)?;
    }
    fs::write(path.as_ref(), builder.into_inner()?)?;
    fs::write(
        sidecar_path(path.as_ref()),
        serde_json::to_vec_pretty(sidecar)?,
    )?;
    Ok(())
}

pub fn write_map(
    path: impl AsRef<Path>,
    values: &BTreeMap<String, u64>,
    sidecar: &FstSidecar,
) -> Result<()> {
    let mut builder = MapBuilder::memory();
    for (key, value) in values {
        builder.insert(key, *value)?;
    }
    fs::write(path.as_ref(), builder.into_inner()?)?;
    fs::write(
        sidecar_path(path.as_ref()),
        serde_json::to_vec_pretty(sidecar)?,
    )?;
    Ok(())
}

fn sidecar_path(path: &Path) -> PathBuf {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.meta.json"))
        .unwrap_or_else(|| "plugin.meta.json".to_string());
    path.with_file_name(filename)
}
