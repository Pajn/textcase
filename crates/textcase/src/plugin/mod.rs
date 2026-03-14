mod inspect;
mod metadata;
mod schema;

pub use inspect::{PluginInspection, inspect_plugin, inspect_plugin_metadata};
pub use metadata::{LicenseMetadata, SourceMetadata};
pub use schema::{PluginKind, PluginMetadata, PluginPayload, PluginSchema, SchemaVersion};
