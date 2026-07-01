//! Promotion derivation for the local MVP.
//!
//! This module owns the semantic transformation from workspace operations to
//! promotion preview/proposal material. It does not discover repositories,
//! persist records, or render CLI output.

#[cfg(test)]
mod tests;

use crate::model::{Change, ChangeStatus, OpKind, PromotionProposal, SemanticDelta, WorkspaceOp};
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};

/// Computed, non-persisted promotion preview material.
///
/// A preview contains only the human-facing semantic delta names and operation
/// count needed for inspection. It intentionally does not carry full semantic
/// deltas or file content.
#[derive(Debug)]
pub struct PromotionPreview {
    /// Semantic delta names that would be produced by proposal creation.
    pub semantic_delta_names: Vec<String>,
    /// Number of workspace operations from the selected change used for derivation.
    pub derived_workspace_operations: usize,
}

/// Computes a non-mutating promotion preview for a selected change.
///
/// The change handle selects workspace operations and supplies lifecycle eligibility.
/// This function does not read or write repository storage.
///
/// # Errors
///
/// Returns an error when the selected change is abandoned.
pub fn preview(change: &Change, ops: &[WorkspaceOp]) -> Result<PromotionPreview> {
    ensure_can_be_proposed(change, "previewed or proposed")?;
    let change_ops = workspace_ops_for_change(&change.handle, ops);
    Ok(PromotionPreview {
        semantic_delta_names: change_ops
            .iter()
            .map(|op| semantic_delta_name(op))
            .collect(),
        derived_workspace_operations: change_ops.len(),
    })
}

/// Creates stored promotion proposal material for a selected change.
///
/// The change handle selects workspace operations and reports errors. This
/// function constructs proposal data only; callers remain responsible for
/// persistence and lifecycle writes.
///
/// # Errors
///
/// Returns an error when the selected change is abandoned or when no workspace
/// operations are recorded for the selected handle.
pub fn create_proposal(
    change: &Change,
    ops: &[WorkspaceOp],
    proposed_at: DateTime<Utc>,
) -> Result<PromotionProposal> {
    ensure_can_be_proposed(change, "proposed")?;
    let change_ops = workspace_ops_for_change(&change.handle, ops);
    if change_ops.is_empty() {
        bail!(
            "no workspace operations recorded for change/{}",
            change.handle
        );
    }
    Ok(PromotionProposal {
        semantic_deltas: semantic_deltas_from_workspace_ops(&change_ops),
        derived_from: change_ops.iter().map(|op| op.id).collect(),
        proposed_at,
    })
}

fn ensure_can_be_proposed(change: &Change, action: &str) -> Result<()> {
    if change.status == ChangeStatus::Abandoned {
        bail!(
            "change/{} is abandoned and cannot be {action}",
            change.handle
        );
    }
    Ok(())
}

fn workspace_ops_for_change<'a>(handle: &str, ops: &'a [WorkspaceOp]) -> Vec<&'a WorkspaceOp> {
    ops.iter().filter(|op| op.change == handle).collect()
}

fn semantic_deltas_from_workspace_ops(ops: &[&WorkspaceOp]) -> Vec<SemanticDelta> {
    ops.iter()
        .map(|op| SemanticDelta {
            name: semantic_delta_name(op),
            kind: op.kind.clone(),
            path: op.path.clone(),
            new_path: op.new_path.clone(),
            content: op.content.clone(),
            class: op.class.clone(),
        })
        .collect()
}

pub(crate) fn semantic_delta_name(op: &WorkspaceOp) -> String {
    match &op.kind {
        OpKind::Add => format!("add {}", op.path),
        OpKind::Update => format!("update {}", op.path),
        OpKind::Remove => format!("remove {}", op.path),
        OpKind::Rename => format!(
            "rename {} to {}",
            op.path,
            op.new_path.as_deref().unwrap_or("<missing new path>")
        ),
    }
}
