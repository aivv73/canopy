use crate::{model::ChangeStatus, storage::LocalStore};
use anyhow::Result;

pub fn run() -> Result<()> {
    let store = LocalStore::discover()?;
    let meta = store.read_meta()?;
    let ops = store.read_workspace_ops()?;
    let tree = store.read_virtual_tree()?;
    let changes = store.load_changes()?;
    let active_change = meta
        .active_change
        .as_ref()
        .map(|handle| store.read_change(handle).map(|change| (handle, change)))
        .transpose()?;

    println!("Canopy repository: {}", meta.name);
    println!("Format: {}", meta.format);

    println!();
    println!("Active change");
    match &active_change {
        Some((handle, change)) => {
            println!("Active change: change/{}", handle);
            println!("  Name: {}", change.name);
            println!("  Handle: change/{}", handle);
            println!("  Status: {}", change.status);
            let active_ops = ops.ops.iter().filter(|op| &op.change == *handle).count();
            println!("  Operations: {}", active_ops);
            println!("Workspace operations: {}", active_ops);
        }
        None => {
            println!("Active change: none");
            println!("  none");
        }
    }

    println!();
    println!("Changes");
    println!("  Active: {}", count_status(&changes, ChangeStatus::Active));
    println!(
        "  Proposed: {}",
        count_status(&changes, ChangeStatus::Proposed)
    );
    println!(
        "  Accepted: {}",
        count_status(&changes, ChangeStatus::Accepted)
    );
    println!(
        "  Abandoned: {} hidden from default change list",
        count_status(&changes, ChangeStatus::Abandoned)
    );
    println!(
        "  Corrective: {}",
        changes
            .iter()
            .filter(|change| change.correction.is_some())
            .count()
    );

    println!();
    println!("Private workspace");
    println!("  Virtual files: {}", tree.files.len());
    println!("  Workspace operations: {}", ops.ops.len());
    // Compatibility line for the original compact status output.
    println!("Workspace operations: {}", ops.ops.len());

    println!();
    println!("Hints");
    if let Some((handle, change)) = &active_change {
        if change.status == ChangeStatus::Accepted {
            println!("  accepted change is still active; run `cnp change finish change/{}` when editing is complete", handle);
        }
        if change.published_at.is_some() || change.disclosed_at.is_some() {
            println!("  published/disclosed change is still active; run `cnp change finish change/{}` when editing is complete", handle);
        }
    }
    println!("  For consistency checks, run `cnp doctor`");
    Ok(())
}

fn count_status(changes: &[crate::model::Change], status: ChangeStatus) -> usize {
    changes
        .iter()
        .filter(|change| change.status == status)
        .count()
}
