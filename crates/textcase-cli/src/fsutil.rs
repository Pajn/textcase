use std::{fs, path::Path};

pub fn ensure_parent_dir(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
