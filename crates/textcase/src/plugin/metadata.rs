use serde::{Deserialize, Serialize};

/// Licensing metadata embedded in prepared files and plugins.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct LicenseMetadata {
    /// Human-readable license name.
    pub name: String,
    /// Short explanation of the obligations that apply to derived plugins.
    pub summary: String,
    /// Optional acknowledgement flag that must be supplied by CLI users.
    #[serde(default)]
    pub acknowledgement_flag: Option<String>,
}

/// Source provenance embedded in prepared files and plugins.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct SourceMetadata {
    /// Stable source identifier such as `wikidata` or `gnd`.
    pub id: String,
    /// Human-readable source name.
    pub display_name: String,
    /// Upstream source URL or project landing page.
    pub url: String,
    /// Source or workflow version used to generate the plugin.
    pub version: String,
    /// Source classification such as `green`, `yellow`, `orange`, or `gray`.
    pub class: String,
}
