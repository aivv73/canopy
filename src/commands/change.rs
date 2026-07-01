use crate::{
    model::{
        Change, ChangeStatus, Correction, CorrectionKind, OpKind, Projection, PromotionProposal,
        PublicationMode, SemanticDelta,
    },
    projection::rebuild_private_virtual_tree,
    storage::{resolve_change_handle, slug, LocalStore},
};
use anyhow::{bail, Result};
use chrono::Utc;

pub fn start(name: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = slug(name);
    if store.change_path(&handle).exists() {
        bail!("change handle already exists: {}", handle);
    }
    let change = Change {
        name: name.into(),
        handle: handle.clone(),
        status: ChangeStatus::Active,
        created_at: Utc::now(),
        proposal: None,
        accepted_at: None,
        published_at: None,
        disclosed_at: None,
        correction: None,
    };
    store.create_change_and_activate(&change)?;
    println!("Started change: {}", name);
    println!("Handle: change/{}", handle);
    Ok(())
}

pub fn correct(target_ref: &str, kind: CorrectionKind, name: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let target_handle = resolve_change_handle(target_ref);
    let target = store.read_change(&target_handle)?;
    if target.status != ChangeStatus::Accepted {
        bail!(
            "change/{} must be accepted before it can be corrected",
            target_handle
        );
    }
    let handle = slug(name);
    if store.change_path(&handle).exists() {
        bail!("change handle already exists: {}", handle);
    }
    let change = Change {
        name: name.into(),
        handle: handle.clone(),
        status: ChangeStatus::Active,
        created_at: Utc::now(),
        proposal: None,
        accepted_at: None,
        published_at: None,
        disclosed_at: None,
        correction: Some(Correction {
            target_change: target_handle.clone(),
            kind,
        }),
    };
    store.create_change_and_activate(&change)?;
    println!("Started corrective change: {}", name);
    println!("Handle: change/{}", handle);
    println!("Correction kind: {}", kind);
    println!("Corrects: change/{}", target_handle);
    println!("Note: no file operations were generated automatically");
    Ok(())
}

pub fn list(all: bool) -> Result<()> {
    let store = LocalStore::discover()?;
    let meta = store.read_meta()?;
    let mut changes = store.load_changes()?;
    changes.sort_by_key(|c| c.created_at);

    println!("Changes");
    if changes.is_empty() {
        println!("  No changes yet.");
        println!("  Start one with `cnp change start <name>`.");
        return Ok(());
    }

    let active_handle = meta.active_change.as_deref();
    if let Some(active) = active_handle.and_then(|handle| {
        changes
            .iter()
            .find(|change| change.handle == handle && change.status != ChangeStatus::Abandoned)
    }) {
        println!();
        println!("Active editing");
        print_change_list_entry(active);
    }

    let other_changes: Vec<_> = changes
        .iter()
        .filter(|change| change.status != ChangeStatus::Abandoned)
        .filter(|change| Some(change.handle.as_str()) != active_handle)
        .collect();
    if !other_changes.is_empty() {
        println!();
        println!("Other changes");
        for change in other_changes {
            print_change_list_entry(change);
        }
    }

    let abandoned: Vec<_> = changes
        .iter()
        .filter(|change| change.status == ChangeStatus::Abandoned)
        .collect();
    if all && !abandoned.is_empty() {
        println!();
        println!("Abandoned");
        for change in abandoned {
            print_change_list_entry(change);
        }
    } else if !all && !abandoned.is_empty() {
        println!();
        println!("Hidden");
        println!(
            "  Abandoned changes: {} hidden; run `cnp change list --all`",
            abandoned.len()
        );
    }
    Ok(())
}

fn print_change_list_entry(change: &Change) {
    println!("  change/{}", change.handle);
    println!("    Name: {}", change.name);
    println!("    Status: {}", change.status);
    println!("    Role: {}", change_list_role(change));
    println!(
        "    Public visibility: {}",
        if change.published_at.is_some() || change.disclosed_at.is_some() {
            "visible"
        } else {
            "not visible"
        }
    );
}

fn change_list_role(change: &Change) -> String {
    match &change.correction {
        Some(correction) => format!("corrective:{}", correction.kind),
        None => "primary".to_string(),
    }
}

pub fn current() -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = store.active_change()?;
    show_by_handle(&store, &handle)
}

pub fn show(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    show_by_handle(&store, &handle)
}

fn show_by_handle(store: &LocalStore, handle: &str) -> Result<()> {
    let change = store.read_change(handle)?;
    let meta = store.read_meta()?;
    let ops = store.read_workspace_ops()?;
    let change_ops: Vec<_> = ops
        .ops
        .iter()
        .filter(|op| op.change == change.handle)
        .collect();
    let active = meta.active_change.as_deref() == Some(&change.handle);

    println!("Identity");
    println!("Change: {}", change.name);
    println!("Handle: change/{}", change.handle);

    println!("Lifecycle");
    println!("Status: {}", change.status);
    println!("Created at: {}", change.created_at.to_rfc3339());
    if let Some(t) = change.accepted_at {
        println!("Accepted at: {}", t.to_rfc3339());
    }
    if let Some(t) = change.published_at {
        println!("Published at: {}", t.to_rfc3339());
    }
    if let Some(t) = change.disclosed_at {
        println!("Disclosed at: {}", t.to_rfc3339());
    }

    println!("Active editing: {}", if active { "yes" } else { "no" });

    println!("Workspace operations");
    println!("Operations: {}", change_ops.len());
    if !change_ops.is_empty() {
        println!("Operation summary:");
        for kind in [OpKind::Add, OpKind::Update, OpKind::Remove, OpKind::Rename] {
            let count = change_ops.iter().filter(|op| op.kind == kind).count();
            if count > 0 {
                println!("  - {}: {}", kind.inspection_label(), count);
            }
        }
        let secret_count = change_ops
            .iter()
            .filter(|op| !op.class.public_safe())
            .count();
        if secret_count > 0 {
            println!("Secret-class operations: {}", secret_count);
        }
    }

    println!("Visibility");
    println!(
        "Public visibility: {}",
        if change.published_at.is_some() || change.disclosed_at.is_some() {
            "visible"
        } else {
            "not visible"
        }
    );

    println!("Correction");
    if let Some(correction) = &change.correction {
        println!("Correction kind: {}", correction.kind);
        println!("Corrects: change/{}", correction.target_change);
    } else {
        println!("Correction: none");
    }

    println!("Promotion proposal");
    if let Some(proposal) = &change.proposal {
        println!(
            "Promotion proposal: {} semantic deltas",
            proposal.semantic_deltas.len()
        );
        println!("Proposed at: {}", proposal.proposed_at.to_rfc3339());
        for delta in &proposal.semantic_deltas {
            println!("  - {}", delta.name);
        }
    } else {
        println!("Promotion proposal: none");
    }
    Ok(())
}

pub fn proposal_show(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let change = store.read_change(&handle)?;
    let Some(proposal) = change.proposal else {
        bail!("change/{} has no promotion proposal", handle);
    };
    println!("Promotion proposal");
    println!("Change: {}", change.name);
    println!("Handle: change/{}", change.handle);
    println!("Status: {}", change.status);
    println!("Proposed at: {}", proposal.proposed_at.to_rfc3339());
    println!();
    println!("Semantic deltas");
    println!("Deltas: {}", proposal.semantic_deltas.len());
    for delta in proposal.semantic_deltas {
        println!("  - {}", delta.name);
    }
    println!();
    println!("Workspace derivation");
    println!(
        "Derived workspace operations: {}",
        proposal.derived_from.len()
    );
    println!("Note: workspace operation IDs are local process history, not projection history.");
    Ok(())
}

pub fn operations(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let change = store.read_change(&handle)?;
    let ops = store.read_workspace_ops()?;
    let change_ops: Vec<_> = ops
        .ops
        .iter()
        .filter(|op| op.change == change.handle)
        .collect();

    println!("Workspace operations");
    println!("Change: {}", change.name);
    println!("Handle: change/{}", change.handle);
    println!("Status: {}", change.status);
    println!("Operations: {}", change_ops.len());

    if change_ops.is_empty() {
        println!();
        println!("No workspace operations recorded for this change.");
        println!("Record one with `cnp file add|update|remove|rename ...`.");
        return Ok(());
    }

    for op in change_ops {
        println!();
        println!("  - {}", workspace_operation_label(op));
        println!("    Class: {}", op.class);
    }
    Ok(())
}

fn workspace_operation_label(op: &crate::model::WorkspaceOp) -> String {
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

pub fn finish(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let change = store.read_change(&handle)?;
    store.finish_active_change(&handle)?;
    println!("Finished active change: {}", change.name);
    println!("Handle: change/{}", change.handle);
    println!("Active change: none");
    Ok(())
}

pub fn abandon(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let mut change = store.read_change(&handle)?;
    if !change.can_be_abandoned() && change.status != ChangeStatus::Abandoned {
        bail!("change/{} is accepted or visible and cannot be abandoned; use `cnp change correct` to create a corrective change", handle);
    }
    let was_abandoned = change.status == ChangeStatus::Abandoned;
    if !was_abandoned {
        change.status = ChangeStatus::Abandoned;
    }
    let meta = store.mark_abandoned_and_clear_active(&change)?;
    rebuild_private_virtual_tree(&store)?;
    if was_abandoned {
        println!("Change already abandoned: {}", change.name);
    } else {
        println!("Abandoned change: {}", change.name);
    }
    println!("Handle: change/{}", change.handle);
    if meta.active_change.is_none() {
        println!("Active change: none");
    }
    Ok(())
}

pub fn propose(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let mut change = store.read_change(&handle)?;
    if change.status == ChangeStatus::Abandoned {
        bail!("change/{} is abandoned and cannot be proposed", handle);
    }
    let ops = store.read_workspace_ops()?;
    let change_ops: Vec<_> = ops
        .ops
        .iter()
        .filter(|op| op.change == handle)
        .cloned()
        .collect();
    if change_ops.is_empty() {
        bail!("no workspace operations recorded for change/{}", handle);
    }
    let deltas = change_ops
        .iter()
        .map(|op| SemanticDelta {
            name: delta_name(op),
            kind: op.kind.clone(),
            path: op.path.clone(),
            new_path: op.new_path.clone(),
            content: op.content.clone(),
            class: op.class.clone(),
        })
        .collect();
    let derived_from = change_ops.iter().map(|op| op.id).collect();
    let proposal = PromotionProposal {
        semantic_deltas: deltas,
        derived_from,
        proposed_at: Utc::now(),
    };
    println!("Promotion proposal created for change: {}", change.name);
    for d in &proposal.semantic_deltas {
        println!("- {}", d.name);
    }
    change.proposal = Some(proposal);
    change.status = ChangeStatus::Proposed;
    store.write_change_proposal(&change)?;
    Ok(())
}

pub fn accept(change_ref: &str) -> Result<()> {
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let mut change = store.read_change(&handle)?;
    if change.status == ChangeStatus::Abandoned {
        bail!("change/{} is abandoned and cannot be accepted", handle);
    }
    if change.proposal.is_none() {
        bail!("change/{} has no promotion proposal", handle);
    }
    change.status = ChangeStatus::Accepted;
    change.accepted_at = Some(Utc::now());
    store.write_change_acceptance(&change)?;
    println!("Accepted change: {}", change.name);
    println!("Handle: change/{}", handle);
    Ok(())
}

pub fn publish(change_ref: &str, to: Projection, mode: PublicationMode) -> Result<()> {
    if to != Projection::Public {
        bail!("MVP only supports publishing/disclosing to public");
    }
    let store = LocalStore::discover()?;
    let handle = resolve_change_handle(change_ref);
    let mut change = store.read_change(&handle)?;
    if change.status != ChangeStatus::Accepted {
        bail!("change/{} must be accepted before publication", handle);
    }
    let now = Utc::now();
    match mode {
        PublicationMode::Disclose => {
            change.disclosed_at = Some(now);
            println!("Disclosed change to public: {}", change.name);
        }
        PublicationMode::Publish => {
            change.published_at = Some(now);
            println!("Published change to public: {}", change.name);
        }
    }
    store.write_change_visibility(&change)?;
    Ok(())
}

fn delta_name(op: &crate::model::WorkspaceOp) -> String {
    workspace_operation_label(op)
}
