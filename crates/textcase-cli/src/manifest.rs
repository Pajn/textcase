use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OutputManifest {
    pub input: String,
    pub output: String,
    pub format: String,
    pub checksum: String,
}
