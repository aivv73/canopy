use crate::{
    model::{Change, ChangeStatus, WorkspaceOps},
    paths::validate_virtual_path,
    projection::private_tree_from_workspace,
    storage::LocalStore,
};
use anyhow::{bail, Result};

pub fn run() -> Result<()> {
    let store = LocalStore::discover()?;
    let mut errors = Vec::new();
    let mut warnings = vec![
        "MVP secret privacy is projection filtering over plaintext local JSON, not encryption"
            .to_string(),
    ];
    let mut hints = Vec::new();
    let mut active_handle = None;
    match store.read_meta() {
        Ok(meta) => {
            if let Some(active) = &meta.active_change {
                active_handle = Some(active.clone());
                if !store.change_path(active).exists() {
                    errors.push(format!("active change does not exist: change/{}", active));
                    hints.push("repair `.canopy/repo.json` or restore the missing change record before continuing local edits".to_string());
                }
            }
        }
        Err(e) => errors.push(e.to_string()),
    }
    let changes = match store.load_changes() {
        Ok(changes) => changes,
        Err(e) => {
            errors.push(format!("cannot read changes: {e}"));
            Vec::new()
        }
    };
    let change_handles: Vec<_> = changes.iter().map(|c| c.handle.clone()).collect();
    if let Some(active) = active_handle {
        if let Some(change) = changes.iter().find(|c| c.handle == active) {
            if change.status == ChangeStatus::Abandoned {
                errors.push(format!(
                    "active change points to abandoned change/{}",
                    active
                ));
                hints.push(format!(
                    "clear the active change with local state repair or start a different change after inspecting change/{}",
                    active
                ));
            }
            if change.status == ChangeStatus::Accepted {
                warnings.push(format!("accepted change is still active; run `cnp change finish change/{}` when editing is complete", active));
                hints.push(format!("run `cnp change finish change/{}` when editing for that accepted change is complete", active));
            }
            if change.published_at.is_some() || change.disclosed_at.is_some() {
                warnings.push(format!("published/disclosed change is still active; run `cnp change finish change/{}` when editing is complete", active));
                hints.push(format!("run `cnp change finish change/{}` to return the repository to no active change", active));
            }
        }
    }
    for change in &changes {
        if change.status == ChangeStatus::Abandoned
            && (change.accepted_at.is_some()
                || change.published_at.is_some()
                || change.disclosed_at.is_some())
        {
            errors.push(format!(
                "abandoned change has accepted/published/disclosed metadata: change/{}",
                change.handle
            ));
            hints.push(format!(
                "inspect change/{} and repair the impossible abandoned lifecycle metadata before relying on history output",
                change.handle
            ));
        }
    }
    validate_corrections(&changes, &mut errors, &mut hints);
    match store.read_workspace_ops() {
        Ok(ops) => validate_ops(&ops, &change_handles, &mut errors),
        Err(e) => errors.push(format!("cannot read workspace operations: {e}")),
    }
    match store.read_virtual_tree() {
        Ok(tree) => {
            for path in tree.files.keys() {
                if let Err(e) = validate_virtual_path(path) {
                    errors.push(format!("virtual tree has invalid path: {e}"));
                }
            }
            match private_tree_from_workspace(&store) {
                Ok(expected) if expected != tree => errors.push(
                    "virtual tree does not match replay of non-abandoned workspace operations"
                        .to_string(),
                ),
                Ok(_) => {}
                Err(e) => errors.push(format!("cannot replay private virtual tree: {e}")),
            }
        }
        Err(e) => errors.push(format!("cannot read virtual tree: {e}")),
    }
    println!("Canopy doctor");
    println!("Checks: local JSON state, change lifecycle, virtual paths, private tree replay");
    if errors.is_empty() {
        println!("Status: healthy");
    } else {
        println!("Status: errors found");
    }
    println!("Errors: {}", errors.len());
    for error in &errors {
        println!("Error: {}", error);
    }
    println!("Warnings: {}", warnings.len());
    for warning in warnings.drain(..) {
        println!("Warning: {}", warning);
    }
    if !hints.is_empty() {
        hints.sort();
        hints.dedup();
        println!("Hints:");
        for hint in hints {
            println!("Hint: {}", hint);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        bail!("doctor found {} error(s)", errors.len())
    }
}

fn validate_corrections(changes: &[Change], errors: &mut Vec<String>, hints: &mut Vec<String>) {
    for change in changes {
        let Some(correction) = &change.correction else {
            continue;
        };
        let Some(target) = changes
            .iter()
            .find(|candidate| candidate.handle == correction.target_change)
        else {
            errors.push(format!(
                "corrective change targets missing change: change/{} corrects change/{}",
                change.handle, correction.target_change
            ));
            hints.push(format!(
                "repair correction metadata on change/{} or restore change/{} before relying on correction history",
                change.handle, correction.target_change
            ));
            continue;
        };
        if target.status != ChangeStatus::Accepted {
            errors.push(format!(
                "corrective change targets non-accepted change: change/{} corrects change/{}",
                change.handle, correction.target_change
            ));
            hints.push(format!(
                "accept the target change or remove correction metadata from change/{}",
                change.handle
            ));
        }
    }
}

fn validate_ops(ops: &WorkspaceOps, change_handles: &[String], errors: &mut Vec<String>) {
    for op in &ops.ops {
        if !change_handles.contains(&op.change) {
            errors.push(format!(
                "workspace operation {} references missing change/{}",
                op.id, op.change
            ));
        }
        if let Err(e) = validate_virtual_path(&op.path) {
            errors.push(format!(
                "workspace operation {} has invalid path: {e}",
                op.id
            ));
        }
        if let Some(new_path) = &op.new_path {
            if let Err(e) = validate_virtual_path(new_path) {
                errors.push(format!(
                    "workspace operation {} has invalid new path: {e}",
                    op.id
                ));
            }
        }
    }
}
