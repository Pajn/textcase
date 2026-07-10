use std::{io, path::PathBuf};

use thiserror::Error;

/// Errors returned by `textcase` when converting text or loading plugins.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("unsupported locale: {0}")]
    UnsupportedLocale(String),
    #[error("plugin schema error: {0}")]
    PluginSchema(String),
    #[error("plugin metadata sidecar not found for {0}")]
    MissingPluginMetadata(PathBuf),
    #[error("invalid plugin path: {0}")]
    InvalidPluginPath(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("fst error: {0}")]
    Fst(#[from] fst::Error),
}

/// Convenience alias for results returned by `textcase`.
pub type Result<T> = std::result::Result<T, Error>;
