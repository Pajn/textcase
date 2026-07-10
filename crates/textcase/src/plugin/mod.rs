mod inspect;
mod metadata;
mod schema;

pub use inspect::{PluginInspection, inspect_plugin, inspect_plugin_metadata};
pub use metadata::{LicenseMetadata, SourceMetadata};
pub use schema::{PluginKind, PluginMetadata, PluginPayload, PluginSchema, SchemaVersion};

/// Computes the SHA-256 checksum, as lowercase hex, of a byte slice.
///
/// This is the same digest stored in [`PluginMetadata::checksum`], exposed so
/// that plugin builders and tooling can compute checksums that match what the
/// library produces and verifies.
pub fn checksum_hex(bytes: &[u8]) -> String {
    crate::util::checksum_hex(bytes)
}
