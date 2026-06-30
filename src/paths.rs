use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub fn normalize_rel(path: &Path) -> Result<String> {
    let path = path
        .to_string_lossy()
        .trim_start_matches("./")
        .replace('\\', "/");
    validate_virtual_path(&path)?;
    Ok(path)
}

pub fn validate_virtual_path(path: &str) -> Result<()> {
    let p = Path::new(path);
    if path.is_empty() || p.is_absolute() {
        bail!(
            "virtual paths must be non-empty repository-relative paths: {}",
            path
        );
    }
    for component in p.components() {
        use std::path::Component;
        match component {
            Component::Normal(part) if part != ".canopy" => {}
            _ => bail!("invalid virtual path: {}", path),
        }
    }
    Ok(())
}

pub fn safe_materialization_path(out_dir: &Path, virtual_path: &str) -> Result<PathBuf> {
    validate_virtual_path(virtual_path)?;
    Ok(out_dir.join(virtual_path))
}
