use super::*;
use crate::model::FileClass;

fn change(status: ChangeStatus) -> Change {
    Change {
        name: "Example".to_string(),
        handle: "example".to_string(),
        status,
        created_at: Utc::now(),
        proposal: None,
        accepted_at: None,
        published_at: None,
        disclosed_at: None,
        correction: None,
    }
}

fn op(
    id: u64,
    kind: OpKind,
    path: &str,
    new_path: Option<&str>,
    content: Option<&str>,
    class: FileClass,
) -> WorkspaceOp {
    op_for("example", id, kind, path, new_path, content, class)
}

fn op_for(
    change: &str,
    id: u64,
    kind: OpKind,
    path: &str,
    new_path: Option<&str>,
    content: Option<&str>,
    class: FileClass,
) -> WorkspaceOp {
    WorkspaceOp {
        id,
        change: change.to_string(),
        kind,
        path: path.to_string(),
        new_path: new_path.map(str::to_string),
        content: content.map(str::to_string),
        class,
        created_at: Utc::now(),
    }
}

#[test]
fn preview_allows_empty_workspace_operations() {
    let preview = preview(&change(ChangeStatus::Active), &[]).unwrap();

    assert!(preview.semantic_delta_names.is_empty());
    assert_eq!(preview.derived_workspace_operations, 0);
}

#[test]
fn proposal_rejects_empty_workspace_operations() {
    let err = match create_proposal(&change(ChangeStatus::Active), &[], Utc::now()) {
        Ok(_) => panic!("empty proposal creation should fail"),
        Err(err) => err,
    };

    assert_eq!(
        err.to_string(),
        "no workspace operations recorded for change/example"
    );
}

#[test]
fn preview_and_proposal_reject_abandoned_changes() {
    let abandoned = change(ChangeStatus::Abandoned);

    let preview_err = preview(&abandoned, &[]).unwrap_err();
    assert_eq!(
        preview_err.to_string(),
        "change/example is abandoned and cannot be previewed or proposed"
    );

    let proposal_err = match create_proposal(&abandoned, &[], Utc::now()) {
        Ok(_) => panic!("abandoned proposal creation should fail"),
        Err(err) => err,
    };
    assert_eq!(
        proposal_err.to_string(),
        "change/example is abandoned and cannot be proposed"
    );
}

#[test]
fn preview_names_match_proposal_delta_names() {
    let ops = vec![
        op(
            1,
            OpKind::Add,
            "README.md",
            None,
            Some("hello"),
            FileClass::PublicSource,
        ),
        op(
            2,
            OpKind::Rename,
            "README.md",
            Some("README2.md"),
            None,
            FileClass::PublicSource,
        ),
    ];

    let preview = preview(&change(ChangeStatus::Active), &ops).unwrap();
    let proposal = create_proposal(&change(ChangeStatus::Active), &ops, Utc::now()).unwrap();
    let proposal_names: Vec<_> = proposal
        .semantic_deltas
        .iter()
        .map(|delta| delta.name.clone())
        .collect();

    assert_eq!(preview.semantic_delta_names, proposal_names);
    assert_eq!(preview.derived_workspace_operations, 2);
}

#[test]
fn promotion_derivation_ignores_other_change_operations() {
    let ops = vec![
        op(
            1,
            OpKind::Add,
            "README.md",
            None,
            Some("hello"),
            FileClass::PublicSource,
        ),
        op_for(
            "other-change",
            2,
            OpKind::Add,
            "other.txt",
            None,
            Some("other"),
            FileClass::PublicSource,
        ),
    ];

    let preview = preview(&change(ChangeStatus::Active), &ops).unwrap();
    let proposal = create_proposal(&change(ChangeStatus::Active), &ops, Utc::now()).unwrap();

    assert_eq!(preview.semantic_delta_names, vec!["add README.md"]);
    assert_eq!(preview.derived_workspace_operations, 1);
    assert_eq!(proposal.derived_from, vec![1]);
    assert_eq!(proposal.semantic_deltas.len(), 1);
    assert_eq!(proposal.semantic_deltas[0].name, "add README.md");
}

#[test]
fn proposal_preserves_full_semantic_delta_fields() {
    let ops = vec![op(
        7,
        OpKind::Update,
        ".env",
        None,
        Some("SECRET=abc\n"),
        FileClass::Secret,
    )];

    let proposal = create_proposal(&change(ChangeStatus::Active), &ops, Utc::now()).unwrap();
    let delta = &proposal.semantic_deltas[0];

    assert_eq!(delta.name, "update .env");
    assert!(matches!(delta.kind, OpKind::Update));
    assert_eq!(delta.path, ".env");
    assert_eq!(delta.new_path, None);
    assert_eq!(delta.content.as_deref(), Some("SECRET=abc\n"));
    assert_eq!(delta.class, FileClass::Secret);
    assert_eq!(proposal.derived_from, vec![7]);
}
