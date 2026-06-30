//! Projection computation for the local MVP.
//!
//! This module computes file entries and replayed virtual-tree state. It does
//! not write files. Public projection replay uses accepted and
//! published/disclosed semantic deltas and must not read current private
//! virtual-tree contents.

use crate::{
    model::{ChangeStatus, FileEntry, OpKind, Projection, VirtualTree, WorkspaceOps},
    storage::LocalStore,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

/// Replays non-abandoned workspace operations into the private virtual-tree cache.
pub fn private_tree_from_workspace(store: &LocalStore) -> Result<VirtualTree> {
    let changes = store.load_changes()?;
    let change_status: BTreeMap<_, _> = changes
        .into_iter()
        .map(|change| (change.handle, change.status))
        .collect();
    let ops: WorkspaceOps = store.read_workspace_ops()?;
    let mut tree = VirtualTree::default();
    for op in ops.ops {
        if change_status.get(&op.change) == Some(&ChangeStatus::Abandoned) {
            continue;
        }
        match op.kind {
            OpKind::Add | OpKind::Update => {
                let content = op.content.ok_or_else(|| {
                    anyhow!(
                        "malformed workspace operation {}: add/update missing content for {}",
                        op.id,
                        op.path
                    )
                })?;
                tree.files.insert(
                    op.path,
                    FileEntry {
                        content,
                        class: op.class,
                        updated_at: op.created_at,
                    },
                );
            }
            OpKind::Remove => {
                tree.files.remove(&op.path);
            }
            OpKind::Rename => {
                let new_path = op.new_path.ok_or_else(|| {
                    anyhow!(
                        "malformed workspace operation {}: rename missing new path for {}",
                        op.id,
                        op.path
                    )
                })?;
                if let Some(mut entry) = tree.files.remove(&op.path) {
                    entry.class = op.class;
                    entry.updated_at = op.created_at;
                    tree.files.insert(new_path, entry);
                }
            }
        }
    }
    Ok(tree)
}

pub fn rebuild_private_virtual_tree(store: &LocalStore) -> Result<()> {
    let tree = private_tree_from_workspace(store)?;
    store.write_virtual_tree(&tree)
}

/// Computes already-filtered materialization entries for a projection.
pub fn materialized_entries(
    store: &LocalStore,
    projection: Projection,
) -> Result<BTreeMap<String, String>> {
    if projection == Projection::Private {
        let tree = store.read_virtual_tree()?;
        return Ok(tree
            .files
            .into_iter()
            .map(|(path, entry)| (path, entry.content))
            .collect());
    }
    let mut entries = BTreeMap::new();
    let mut changes = store.load_changes()?;
    changes.sort_by_key(|c| c.created_at);
    for change in changes {
        if change.status != ChangeStatus::Accepted
            || (change.published_at.is_none() && change.disclosed_at.is_none())
        {
            continue;
        }
        let Some(proposal) = change.proposal else {
            continue;
        };
        for delta in proposal.semantic_deltas {
            if !delta.class.public_safe() {
                continue;
            }
            match delta.kind {
                OpKind::Add | OpKind::Update => {
                    if let Some(content) = delta.content {
                        entries.insert(delta.path, content);
                    }
                }
                OpKind::Remove => {
                    entries.remove(&delta.path);
                }
                OpKind::Rename => {
                    if let Some(new_path) = delta.new_path {
                        if let Some(content) = entries.remove(&delta.path) {
                            entries.insert(new_path, content);
                        }
                    }
                }
            }
        }
    }
    Ok(entries)
}
