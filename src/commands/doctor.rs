use crate::{
    model::{ChangeStatus, WorkspaceOps},
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
    let mut active_handle = None;
    match store.read_meta() {
        Ok(meta) => {
            if let Some(active) = &meta.active_change {
                active_handle = Some(active.clone());
                if !store.change_path(active).exists() {
                    errors.push(format!("active change does not exist: change/{}", active));
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
            }
            if change.status == ChangeStatus::Accepted {
                warnings.push(format!("accepted change is still active; run `cnp change finish change/{}` when editing is complete", active));
            }
            if change.published_at.is_some() || change.disclosed_at.is_some() {
                warnings.push(format!("published/disclosed change is still active; run `cnp change finish change/{}` when editing is complete", active));
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
        }
    }
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
    if errors.is_empty() {
        println!("Status: healthy");
    } else {
        println!("Status: errors found");
        for error in &errors {
            println!("Error: {}", error);
        }
    }
    for warning in warnings.drain(..) {
        println!("Warning: {}", warning);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        bail!("doctor found {} error(s)", errors.len())
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
