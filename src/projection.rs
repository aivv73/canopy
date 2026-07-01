//! Projection computation for the local MVP.
//!
//! This module computes file entries and replayed virtual-tree state. It does
//! not write files. Public projection replay uses accepted and
//! published/disclosed semantic deltas and must not read current private
//! virtual-tree contents.

use crate::{
    model::{
        Change, ChangeStatus, Correction, FileEntry, OpKind, Projection, SemanticDelta,
        VirtualTree, WorkspaceOps,
    },
    storage::LocalStore,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};

/// A change as it appears in a computed projection history view.
pub struct ProjectedChange {
    pub name: String,
    pub handle: String,
    pub status: ChangeStatus,
    pub visible_at: Option<DateTime<Utc>>,
    pub correction: Option<Correction>,
    pub deltas: Vec<SemanticDelta>,
}

pub struct CorrectionInvariantError {
    pub corrective_change: String,
    pub target_change: String,
    pub kind: CorrectionInvariantErrorKind,
}

pub enum CorrectionInvariantErrorKind {
    MissingTarget,
    NonAcceptedTarget,
}

impl CorrectionInvariantError {
    pub fn message(&self) -> String {
        match self.kind {
            CorrectionInvariantErrorKind::MissingTarget => format!(
                "corrective change targets missing change: change/{} corrects change/{}",
                self.corrective_change, self.target_change
            ),
            CorrectionInvariantErrorKind::NonAcceptedTarget => format!(
                "corrective change targets non-accepted change: change/{} corrects change/{}",
                self.corrective_change, self.target_change
            ),
        }
    }
}

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
    store.write_private_virtual_tree_cache(&tree)
}

/// Computes accepted semantic history visible through the requested projection.
pub fn projected_history(
    store: &LocalStore,
    projection: Projection,
) -> Result<Vec<ProjectedChange>> {
    let mut changes = store.load_changes()?;
    changes.sort_by_key(|c| c.created_at);
    let mut projected = Vec::new();
    for change in changes {
        if !change_lifecycle_visible_in_history(&change, projection) {
            continue;
        }
        let Some(proposal) = &change.proposal else {
            continue;
        };
        let deltas = visible_semantic_deltas(&proposal.semantic_deltas, projection);
        if deltas.is_empty() {
            continue;
        }
        let visible_at = match projection {
            Projection::Public => change
                .disclosed_at
                .or(change.published_at)
                .or(change.accepted_at),
            Projection::Private => change.accepted_at,
        };
        projected.push(ProjectedChange {
            name: change.name,
            handle: change.handle,
            status: change.status,
            visible_at,
            correction: change.correction,
            deltas,
        });
    }
    let visible_handles: BTreeSet<_> = projected
        .iter()
        .map(|change| change.handle.clone())
        .collect();
    if projection == Projection::Public {
        for change in &mut projected {
            if let Some(correction) = &change.correction {
                if !visible_handles.contains(&correction.target_change) {
                    change.correction = None;
                }
            }
        }
    }
    Ok(projected)
}

/// Returns whether a change's lifecycle permits it to appear in projection history.
pub(crate) fn change_lifecycle_visible_in_history(change: &Change, projection: Projection) -> bool {
    change.status == ChangeStatus::Accepted
        && (projection == Projection::Private
            || change.published_at.is_some()
            || change.disclosed_at.is_some())
}

/// Filters semantic deltas according to projection visibility rules.
pub(crate) fn visible_semantic_deltas(
    deltas: &[SemanticDelta],
    projection: Projection,
) -> Vec<SemanticDelta> {
    deltas
        .iter()
        .filter(|d| projection == Projection::Private || d.class.public_safe())
        .cloned()
        .collect()
}

/// Validates correction metadata target existence and accepted lifecycle.
pub fn validate_correction_invariants(changes: &[Change]) -> Vec<CorrectionInvariantError> {
    let mut errors = Vec::new();
    for change in changes {
        let Some(correction) = &change.correction else {
            continue;
        };
        let Some(target) = changes
            .iter()
            .find(|candidate| candidate.handle == correction.target_change)
        else {
            errors.push(CorrectionInvariantError {
                corrective_change: change.handle.clone(),
                target_change: correction.target_change.clone(),
                kind: CorrectionInvariantErrorKind::MissingTarget,
            });
            continue;
        };
        if target.status != ChangeStatus::Accepted {
            errors.push(CorrectionInvariantError {
                corrective_change: change.handle.clone(),
                target_change: correction.target_change.clone(),
                kind: CorrectionInvariantErrorKind::NonAcceptedTarget,
            });
        }
    }
    errors
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
        if !change_lifecycle_visible_in_history(&change, Projection::Public) {
            continue;
        }
        let Some(proposal) = change.proposal else {
            continue;
        };
        for delta in visible_semantic_deltas(&proposal.semantic_deltas, Projection::Public) {
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
