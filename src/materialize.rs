//! Filesystem materialization for already-computed projection entries.
//!
//! This module owns output directory marker checks and writes. It does not
//! decide visibility or inspect Canopy history.

use crate::paths::safe_materialization_path;
use anyhow::{bail, Result};
use std::{collections::BTreeMap, fs, path::Path};

const MATERIALIZATION_MARKER: &str = "canopy-mvp-materialization-v1\n";

/// Writes already-computed entries into a marker-protected directory.
pub fn materialize_entries(entries: BTreeMap<String, String>, out_dir: &Path) -> Result<()> {
    prepare_materialization_dir(out_dir)?;
    for (path, content) in entries {
        let dest = safe_materialization_path(out_dir, &path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, content)?;
    }
    fs::write(out_dir.join(".canopy-materialized"), MATERIALIZATION_MARKER)?;
    Ok(())
}

fn prepare_materialization_dir(out_dir: &Path) -> Result<()> {
    if out_dir.exists() {
        if !out_dir.is_dir() {
            bail!(
                "materialization target exists but is not a directory: {}",
                out_dir.display()
            );
        }
        let marker = out_dir.join(".canopy-materialized");
        let mut entries = fs::read_dir(out_dir)?;
        if marker.exists() {
            let marker_content = fs::read_to_string(&marker)?;
            if marker_content != MATERIALIZATION_MARKER {
                bail!(
                    "materialization target has an invalid Canopy marker: {}",
                    out_dir.display()
                );
            }
            for entry in fs::read_dir(out_dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        } else if entries.next().is_some() {
            bail!(
                "materialization target is not empty and was not created by Canopy: {}",
                out_dir.display()
            );
        }
    }
    fs::create_dir_all(out_dir)?;
    Ok(())
}
