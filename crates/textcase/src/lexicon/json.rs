use crate::{Result, plugin::PluginSchema};

pub fn load_json_plugin(bytes: &[u8]) -> Result<PluginSchema> {
    Ok(serde_json::from_slice(bytes)?)
}
