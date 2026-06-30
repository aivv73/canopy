//! Local JSON persistence boundary for the MVP `.canopy/` directory.
//!
//! `LocalStore` owns repository discovery, storage-format validation, and JSON
//! file access. It preserves the MVP's current per-file write-then-rename
//! behavior but does not provide cross-file transactions.

use crate::model::{Change, RepoMeta, VirtualTree, WorkspaceOps};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const CANOPY_DIR: &str = ".canopy";
pub const FORMAT: &str = "canopy-mvp-1";

#[derive(Clone)]
pub struct LocalStore {
    root: PathBuf,
}

impl LocalStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        for dir in cwd.ancestors() {
            let c = dir.join(CANOPY_DIR);
            if c.is_dir() {
                return Ok(Self::new(c));
            }
        }
        bail!("not inside a Canopy repository")
    }

    pub fn change_path(&self, handle: &str) -> PathBuf {
        self.root.join("changes").join(format!("{}.json", handle))
    }
    pub fn repo_path(&self) -> PathBuf {
        self.root.join("repo.json")
    }
    pub fn virtual_tree_path(&self) -> PathBuf {
        self.root.join("virtual-tree.json")
    }
    pub fn workspace_ops_path(&self) -> PathBuf {
        self.root.join("workspace-ops.json")
    }

    pub fn read_meta(&self) -> Result<RepoMeta> {
        let meta: RepoMeta = read_json(&self.repo_path())?;
        if meta.format != FORMAT {
            bail!(
                "unsupported Canopy storage format `{}`; this cnp supports `{}`",
                meta.format,
                FORMAT
            );
        }
        Ok(meta)
    }
    pub fn write_meta(&self, meta: &RepoMeta) -> Result<()> {
        write_json(&self.repo_path(), meta)
    }
    pub fn read_change(&self, handle: &str) -> Result<Change> {
        read_json(&self.change_path(handle))
    }
    pub fn write_change(&self, change: &Change) -> Result<()> {
        write_json(&self.change_path(&change.handle), change)
    }
    pub fn load_changes(&self) -> Result<Vec<Change>> {
        let mut out = vec![];
        for e in fs::read_dir(self.root.join("changes"))? {
            let e = e?;
            if e.path().extension().and_then(|s| s.to_str()) == Some("json") {
                out.push(read_json(&e.path())?);
            }
        }
        Ok(out)
    }
    pub fn read_workspace_ops(&self) -> Result<WorkspaceOps> {
        read_json(&self.workspace_ops_path())
    }
    pub fn write_workspace_ops(&self, ops: &WorkspaceOps) -> Result<()> {
        write_json(&self.workspace_ops_path(), ops)
    }
    pub fn read_virtual_tree(&self) -> Result<VirtualTree> {
        read_json(&self.virtual_tree_path())
    }
    pub fn write_virtual_tree(&self, tree: &VirtualTree) -> Result<()> {
        write_json(&self.virtual_tree_path(), tree)
    }

    pub fn active_change(&self) -> Result<String> {
        let meta = self.read_meta()?;
        meta.active_change
            .ok_or_else(|| anyhow::anyhow!("no active change; run `cnp change start <name>` first"))
    }

    pub fn append_workspace_op(&self, mut op: crate::model::WorkspaceOp) -> Result<()> {
        let mut ops = self.read_workspace_ops()?;
        op.id = ops.ops.iter().map(|op| op.id).max().unwrap_or(0) + 1;
        ops.ops.push(op);
        self.write_workspace_ops(&ops)
    }
}

pub fn init_layout(path: Option<PathBuf>) -> Result<(LocalStore, PathBuf)> {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    if root.exists() && !root.is_dir() {
        bail!(
            "init path exists but is not a directory: {}",
            root.display()
        );
    }
    fs::create_dir_all(&root)?;
    let canopy = root.join(CANOPY_DIR);
    if canopy.exists() {
        bail!("Canopy repository already exists at {}", canopy.display());
    }
    fs::create_dir(&canopy)?;
    fs::create_dir(canopy.join("changes"))?;
    let store = LocalStore::new(canopy);
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("canopy-repo")
        .to_string();
    store.write_meta(&RepoMeta {
        name,
        format: FORMAT.into(),
        active_change: None,
    })?;
    store.write_virtual_tree(&VirtualTree::default())?;
    store.write_workspace_ops(&WorkspaceOps::default())?;
    Ok((store, root))
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse JSON state {}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))
}

pub fn slug(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn resolve_change_handle(s: &str) -> String {
    let raw = s.strip_prefix("change/").unwrap_or(s);
    slug(raw)
}
