use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct LicenseMetadata {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub acknowledgement_flag: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SourceMetadata {
    pub id: String,
    pub display_name: String,
    pub url: String,
    pub version: String,
    pub class: String,
}
