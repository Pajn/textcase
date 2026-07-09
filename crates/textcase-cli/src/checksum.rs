use std::{fs, path::Path};

use textcase::plugin::checksum_hex;

pub fn file_checksum(path: impl AsRef<Path>) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    Ok(checksum_hex(&bytes))
}
