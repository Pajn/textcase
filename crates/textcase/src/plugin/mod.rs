mod inspect;
mod metadata;
mod schema;

pub use inspect::{PluginInspection, inspect_plugin};
pub use metadata::{LicenseMetadata, SourceMetadata};
pub use schema::{PluginKind, PluginMetadata, PluginPayload, PluginSchema, SchemaVersion};
