use crate::{
    cli::{FileAdd, FileRemove, FileRename, FileUpdate},
    model::{FileClass, FileEntry, OpKind, WorkspaceOp},
    paths::normalize_rel,
    storage::LocalStore,
};
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use std::fs;

pub fn add(args: FileAdd) -> Result<()> {
    let store = LocalStore::discover()?;
    let change = store.active_change()?;
    let rel = normalize_rel(&args.path)?;
    let content =
        fs::read_to_string(&args.path).with_context(|| format!("read {}", args.path.display()))?;
    let now = Utc::now();
    let mut tree = store.read_virtual_tree()?;
    tree.files.insert(
        rel.clone(),
        FileEntry {
            content: content.clone(),
            class: args.class.clone(),
            updated_at: now,
        },
    );
    store.record_file_operation(
        &tree,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Add,
            path: rel.clone(),
            new_path: None,
            content: Some(content),
            class: args.class.clone(),
            created_at: now,
        },
    )?;
    println!("Added file: {}", rel);
    println!("Class: {}", args.class);
    warn_secret(&args.class);
    Ok(())
}

pub fn update(args: FileUpdate) -> Result<()> {
    let store = LocalStore::discover()?;
    let change = store.active_change()?;
    let rel = normalize_rel(&args.path)?;
    let content =
        fs::read_to_string(&args.path).with_context(|| format!("read {}", args.path.display()))?;
    let now = Utc::now();
    let mut tree = store.read_virtual_tree()?;
    let existing = tree
        .files
        .get(&rel)
        .ok_or_else(|| anyhow!("cannot update unknown virtual file: {}", rel))?;
    let class = args.class.unwrap_or_else(|| existing.class.clone());
    if matches!(existing.class, FileClass::Secret) && class.public_safe() {
        bail!(
            "cannot reclassify secret file as public-safe during update: {}",
            rel
        );
    }
    tree.files.insert(
        rel.clone(),
        FileEntry {
            content: content.clone(),
            class: class.clone(),
            updated_at: now,
        },
    );
    store.record_file_operation(
        &tree,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Update,
            path: rel.clone(),
            new_path: None,
            content: Some(content),
            class: class.clone(),
            created_at: now,
        },
    )?;
    println!("Updated file: {}", rel);
    println!("Class: {}", class);
    warn_secret(&class);
    Ok(())
}

pub fn remove(args: FileRemove) -> Result<()> {
    let store = LocalStore::discover()?;
    let change = store.active_change()?;
    let rel = normalize_rel(&args.path)?;
    let now = Utc::now();
    let mut tree = store.read_virtual_tree()?;
    let removed = tree
        .files
        .remove(&rel)
        .ok_or_else(|| anyhow!("cannot remove unknown virtual file: {}", rel))?;
    store.record_file_operation(
        &tree,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Remove,
            path: rel.clone(),
            new_path: None,
            content: None,
            class: removed.class.clone(),
            created_at: now,
        },
    )?;
    println!("Removed file: {}", rel);
    println!("Class: {}", removed.class);
    warn_secret(&removed.class);
    Ok(())
}

pub fn rename(args: FileRename) -> Result<()> {
    let store = LocalStore::discover()?;
    let change = store.active_change()?;
    let old = normalize_rel(&args.old_path)?;
    let new = normalize_rel(&args.new_path)?;
    if old == new {
        bail!("rename source and destination are the same: {}", old);
    }
    let now = Utc::now();
    let mut tree = store.read_virtual_tree()?;
    if tree.files.contains_key(&new) {
        bail!("cannot rename over existing virtual file: {}", new);
    }
    let mut entry = tree
        .files
        .remove(&old)
        .ok_or_else(|| anyhow!("cannot rename unknown virtual file: {}", old))?;
    if matches!(entry.class, FileClass::Secret)
        && args.class.as_ref().is_some_and(FileClass::public_safe)
    {
        bail!(
            "cannot reclassify secret file as public-safe during rename: {}",
            old
        );
    }
    if let Some(class) = args.class {
        entry.class = class;
    }
    entry.updated_at = now;
    let class = entry.class.clone();
    tree.files.insert(new.clone(), entry);
    store.record_file_operation(
        &tree,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Rename,
            path: old.clone(),
            new_path: Some(new.clone()),
            content: None,
            class: class.clone(),
            created_at: now,
        },
    )?;
    println!("Renamed file: {} -> {}", old, new);
    println!("Class: {}", class);
    warn_secret(&class);
    Ok(())
}

fn warn_secret(class: &FileClass) {
    if matches!(class, FileClass::Secret) {
        println!("Warning: Canopy MVP stores secret files in plaintext under .canopy/. They are hidden from public projections but not encrypted at rest.");
    }
}
