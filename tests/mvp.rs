use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn mvp_public_private_projection_flow() {
    let temp = tempdir().unwrap();
    let bin = assert_cmd::cargo::cargo_bin("cnp");

    Command::new(&bin)
        .args(["init", "demo"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Canopy MVP"));
    let demo = temp.path().join("demo");
    fs::write(demo.join("README.md"), "hello\n").unwrap();
    fs::write(demo.join(".env"), "SECRET=abc\n").unwrap();
    fs::write(demo.join(".env.example"), "SECRET=\n").unwrap();

    Command::new(&bin)
        .args(["change", "start", "Initial project files"])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args(["file", "add", "README.md"])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args(["file", "add", ".env", "--class", "secret"])
        .current_dir(&demo)
        .assert()
        .success()
        .stdout(predicate::str::contains("not encrypted at rest"));
    Command::new(&bin)
        .args(["file", "add", ".env.example", "--class", "config-template"])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args(["change", "propose", "Initial project files"])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args(["change", "accept", "Initial project files"])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args([
            "change",
            "publish",
            "Initial project files",
            "--to",
            "public",
        ])
        .current_dir(&demo)
        .assert()
        .success();

    let public = temp.path().join("public");
    let private = temp.path().join("private");
    Command::new(&bin)
        .args([
            "projection",
            "materialize",
            "public",
            public.to_str().unwrap(),
        ])
        .current_dir(&demo)
        .assert()
        .success();
    Command::new(&bin)
        .args([
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ])
        .current_dir(&demo)
        .assert()
        .success();

    assert!(public.join("README.md").exists());
    assert!(public.join(".env.example").exists());
    assert!(!public.join(".env").exists());
    assert!(private.join("README.md").exists());
    assert!(private.join(".env.example").exists());
    assert!(private.join(".env").exists());

    let public_history = Command::new(&bin)
        .args(["history", "--projection", "public"])
        .current_dir(&demo)
        .output()
        .unwrap();
    let public_history = String::from_utf8(public_history.stdout).unwrap();
    assert!(public_history.contains("README.md"));
    assert!(public_history.contains(".env.example"));
    assert!(!public_history
        .lines()
        .any(|line| line.trim() == "- add .env"));

    let private_history = Command::new(&bin)
        .args(["history", "--projection", "private"])
        .current_dir(&demo)
        .output()
        .unwrap();
    let private_history = String::from_utf8(private_history.stdout).unwrap();
    assert!(private_history
        .lines()
        .any(|line| line.trim() == "- add .env"));
}

fn run(cwd: &std::path::Path, args: &[&str]) {
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(cwd)
        .args(args)
        .assert()
        .success();
}

#[test]
fn public_materialization_only_includes_published_accepted_changes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);

    fs::write(repo.join("published.txt"), "published\n").unwrap();
    fs::write(repo.join("draft.txt"), "draft\n").unwrap();

    run(&repo, &["change", "start", "Published file"]);
    run(&repo, &["file", "add", "published.txt"]);
    run(&repo, &["change", "propose", "Published file"]);
    run(&repo, &["change", "accept", "Published file"]);
    run(
        &repo,
        &["change", "publish", "Published file", "--to", "public"],
    );

    run(&repo, &["change", "start", "Draft file"]);
    run(&repo, &["file", "add", "draft.txt"]);

    let public_dir = temp.path().join("public");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "public",
            public_dir.to_str().unwrap(),
        ],
    );

    assert!(public_dir.join("published.txt").exists());
    assert!(!public_dir.join("draft.txt").exists());
}

#[test]
fn file_add_rejects_paths_outside_virtual_tree() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    run(&repo, &["change", "start", "Bad path"]);

    let output = Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["file", "add", "../outside.txt"])
        .output()
        .expect("run cnp");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid virtual path"));
}

#[test]
fn materialization_rejects_invalid_marker() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);

    let out = temp.path().join("public");
    fs::create_dir(&out).unwrap();
    fs::write(out.join(".canopy-materialized"), "not-canopy\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["projection", "materialize", "public", out.to_str().unwrap()])
        .output()
        .expect("run cnp");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid Canopy marker"));
}

#[test]
fn status_and_change_inspection_show_local_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();
    run(&repo, &["change", "start", "Inspect me"]);
    run(&repo, &["file", "add", "README.md"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active change: change/inspect-me"))
        .stdout(predicate::str::contains("Workspace operations: 1"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inspect me"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "current"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity"))
        .stdout(predicate::str::contains("Handle: change/inspect-me"))
        .stdout(predicate::str::contains("Lifecycle"))
        .stdout(predicate::str::contains("Active editing: yes"))
        .stdout(predicate::str::contains("Workspace operations"))
        .stdout(predicate::str::contains("Operations: 1"))
        .stdout(predicate::str::contains("Operation summary:"))
        .stdout(predicate::str::contains("Promotion proposal: none"));
}

#[test]
fn richer_inspection_outputs_explain_change_history_and_doctor_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();
    fs::write(repo.join(".env"), "SECRET=abc\n").unwrap();

    run(&repo, &["change", "start", "Richer inspection"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["file", "add", ".env", "--class", "secret"]);
    run(&repo, &["change", "propose", "Richer inspection"]);
    run(&repo, &["change", "accept", "Richer inspection"]);
    run(
        &repo,
        &["change", "publish", "Richer inspection", "--to", "public"],
    );

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "show", "Richer inspection"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity"))
        .stdout(predicate::str::contains("Lifecycle"))
        .stdout(predicate::str::contains("Active editing: yes"))
        .stdout(predicate::str::contains("Operations: 2"))
        .stdout(predicate::str::contains("Secret-class operations: 1"))
        .stdout(predicate::str::contains("Public visibility: visible"))
        .stdout(predicate::str::contains(
            "Promotion proposal: 2 semantic deltas",
        ));

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["history", "--projection", "public"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Projection history"))
        .stdout(predicate::str::contains("Projection: public"))
        .stdout(predicate::str::contains(
            "History kind: accepted semantic deltas",
        ))
        .stdout(predicate::str::contains("Visibility: public"))
        .stdout(predicate::str::contains("Changes shown: 1"))
        .stdout(predicate::str::contains("add .env").not());

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checks: local JSON state"))
        .stdout(predicate::str::contains("Errors: 0"))
        .stdout(predicate::str::contains("Warnings:"))
        .stdout(predicate::str::contains("Hint: run `cnp change finish"));
}

#[test]
fn update_remove_and_rename_flow_through_projections() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "v1\n").unwrap();
    fs::write(repo.join("notes.txt"), "notes\n").unwrap();
    fs::write(repo.join("secret.txt"), "SECRET\n").unwrap();

    run(&repo, &["change", "start", "Lifecycle"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["file", "add", "notes.txt"]);
    run(&repo, &["file", "add", "secret.txt", "--class", "secret"]);
    fs::write(repo.join("README.md"), "v2\n").unwrap();
    run(&repo, &["file", "update", "README.md"]);
    run(&repo, &["file", "rename", "README.md", "README2.md"]);
    run(&repo, &["file", "remove", "notes.txt"]);
    run(&repo, &["change", "propose", "Lifecycle"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "proposal", "Lifecycle"])
        .assert()
        .success()
        .stdout(predicate::str::contains("update README.md"))
        .stdout(predicate::str::contains("rename README.md to README2.md"))
        .stdout(predicate::str::contains("remove notes.txt"));
    run(&repo, &["change", "accept", "Lifecycle"]);
    run(&repo, &["change", "publish", "Lifecycle", "--to", "public"]);

    let public = temp.path().join("public-life");
    let private = temp.path().join("private-life");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "public",
            public.to_str().unwrap(),
        ],
    );
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read_to_string(public.join("README2.md")).unwrap(),
        "v2\n"
    );
    assert!(!public.join("README.md").exists());
    assert!(!public.join("notes.txt").exists());
    assert!(!public.join("secret.txt").exists());
    assert!(private.join("secret.txt").exists());
}

#[test]
fn doctor_reports_health_and_storage_errors() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: healthy"))
        .stdout(predicate::str::contains("plaintext local JSON"));

    fs::write(
        repo.join(".canopy/repo.json"),
        "{\"name\":\"demo\",\"format\":\"future\",\"active_change\":null}\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "unsupported Canopy storage format",
        ));
}

#[test]
fn public_materialization_does_not_read_unpublished_private_tree_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "public\n").unwrap();

    run(&repo, &["change", "start", "Published readme"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["change", "propose", "Published readme"]);
    run(&repo, &["change", "accept", "Published readme"]);
    run(
        &repo,
        &["change", "publish", "Published readme", "--to", "public"],
    );

    fs::write(repo.join("README.md"), "draft secret\n").unwrap();
    run(&repo, &["change", "start", "Unpublished secret edit"]);
    run(&repo, &["file", "update", "README.md", "--class", "secret"]);

    let public = temp.path().join("public-stable");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "public",
            public.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read_to_string(public.join("README.md")).unwrap(),
        "public\n"
    );

    let private = temp.path().join("private-draft");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read_to_string(private.join("README.md")).unwrap(),
        "draft secret\n"
    );
}

#[test]
fn finish_clears_active_change_without_changing_history_or_materialization() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();

    run(&repo, &["change", "start", "Finish me"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["change", "propose", "Finish me"]);
    run(&repo, &["change", "accept", "Finish me"]);
    run(&repo, &["change", "publish", "Finish me", "--to", "public"]);
    run(&repo, &["change", "finish", "Finish me"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active change: none"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "show", "Finish me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Change: Finish me"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["history", "--projection", "public"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add README.md"));

    let public = temp.path().join("finished-public");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "public",
            public.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read_to_string(public.join("README.md")).unwrap(),
        "hello\n"
    );
}

#[test]
fn no_active_change_errors_are_clear_after_finish() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();
    run(&repo, &["change", "start", "Done"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["change", "finish", "Done"]);

    for args in [
        vec!["file", "add", "README.md"],
        vec!["file", "update", "README.md"],
        vec!["file", "remove", "README.md"],
        vec!["file", "rename", "README.md", "README2.md"],
    ] {
        Command::new(env!("CARGO_BIN_EXE_cnp"))
            .current_dir(&repo)
            .args(args)
            .assert()
            .failure()
            .stderr(predicate::str::contains("no active change"))
            .stderr(predicate::str::contains("cnp change start"));
    }
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "current"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no active change"));
}

#[test]
fn finish_refuses_non_active_change_and_second_change_gets_ops() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("a.txt"), "a\n").unwrap();
    fs::write(repo.join("b.txt"), "b\n").unwrap();

    run(&repo, &["change", "start", "First"]);
    run(&repo, &["file", "add", "a.txt"]);
    run(&repo, &["change", "finish", "First"]);
    run(&repo, &["change", "start", "Second"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "finish", "First"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("change/second is active"));
    run(&repo, &["file", "add", "b.txt"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active change: change/second"))
        .stdout(predicate::str::contains("Workspace operations: 1"));
}

#[test]
fn doctor_reports_active_change_lifecycle_problems() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();
    run(&repo, &["change", "start", "Accepted active"]);
    run(&repo, &["file", "add", "README.md"]);
    run(&repo, &["change", "propose", "Accepted active"]);
    run(&repo, &["change", "accept", "Accepted active"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("accepted change is still active"));

    fs::write(
        repo.join(".canopy/repo.json"),
        "{\"name\":\"demo\",\"format\":\"canopy-mvp-1\",\"active_change\":\"missing\"}\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("active change does not exist"));
}

#[test]
fn abandon_hides_change_by_default_and_cleans_added_file() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("bad.txt"), "bad\n").unwrap();

    run(&repo, &["change", "start", "Bad idea"]);
    run(&repo, &["file", "add", "bad.txt"]);
    run(&repo, &["change", "abandon", "Bad idea"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active change: none"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Bad idea").not());
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Bad idea"))
        .stdout(predicate::str::contains("abandoned"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "show", "Bad idea"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: abandoned"));

    let private = temp.path().join("abandoned-private");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ],
    );
    assert!(!private.join("bad.txt").exists());
}

#[test]
fn abandon_retains_proposal_but_excludes_history() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("maybe.txt"), "maybe\n").unwrap();

    run(&repo, &["change", "start", "Maybe"]);
    run(&repo, &["file", "add", "maybe.txt"]);
    run(&repo, &["change", "propose", "Maybe"]);
    run(&repo, &["change", "abandon", "Maybe"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "show", "Maybe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: abandoned"))
        .stdout(predicate::str::contains("add maybe.txt"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["history", "--projection", "private"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Maybe").not());
}

#[test]
fn abandon_refuses_accepted_published_and_disclosed_changes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("a.txt"), "a\n").unwrap();
    fs::write(repo.join("b.txt"), "b\n").unwrap();

    run(&repo, &["change", "start", "Accepted"]);
    run(&repo, &["file", "add", "a.txt"]);
    run(&repo, &["change", "propose", "Accepted"]);
    run(&repo, &["change", "accept", "Accepted"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "abandon", "Accepted"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be abandoned"));
    run(&repo, &["change", "publish", "Accepted", "--to", "public"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "abandon", "Accepted"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be abandoned"));

    run(&repo, &["change", "finish", "Accepted"]);
    run(&repo, &["change", "start", "Disclosed"]);
    run(&repo, &["file", "add", "b.txt"]);
    run(&repo, &["change", "propose", "Disclosed"]);
    run(&repo, &["change", "accept", "Disclosed"]);
    run(
        &repo,
        &["change", "disclose", "Disclosed", "--to", "public"],
    );
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "abandon", "Disclosed"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be abandoned"));
}

#[test]
fn abandon_replays_private_tree_without_update_remove_or_rename_effects() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("stable.txt"), "v1\n").unwrap();
    fs::write(repo.join("remove.txt"), "keep\n").unwrap();
    fs::write(repo.join("rename.txt"), "name\n").unwrap();

    run(&repo, &["change", "start", "Base"]);
    run(&repo, &["file", "add", "stable.txt"]);
    run(&repo, &["file", "add", "remove.txt"]);
    run(&repo, &["file", "add", "rename.txt"]);
    run(&repo, &["change", "finish", "Base"]);

    fs::write(repo.join("stable.txt"), "v2\n").unwrap();
    run(&repo, &["change", "start", "Bad update"]);
    run(&repo, &["file", "update", "stable.txt"]);
    run(&repo, &["file", "remove", "remove.txt"]);
    run(&repo, &["file", "rename", "rename.txt", "renamed.txt"]);
    run(&repo, &["change", "abandon", "Bad update"]);

    let private = temp.path().join("replayed-private");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::read_to_string(private.join("stable.txt")).unwrap(),
        "v1\n"
    );
    assert_eq!(
        fs::read_to_string(private.join("remove.txt")).unwrap(),
        "keep\n"
    );
    assert_eq!(
        fs::read_to_string(private.join("rename.txt")).unwrap(),
        "name\n"
    );
    assert!(!private.join("renamed.txt").exists());
}

#[test]
fn doctor_understands_abandoned_lifecycle_invariants() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("bad.txt"), "bad\n").unwrap();
    run(&repo, &["change", "start", "Bad"]);
    run(&repo, &["file", "add", "bad.txt"]);
    run(&repo, &["change", "abandon", "Bad"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .success();

    let change_path = repo.join(".canopy/changes/bad.json");
    let impossible = fs::read_to_string(&change_path).unwrap().replace(
        "\"accepted_at\": null",
        "\"accepted_at\": \"2026-06-30T00:00:00Z\"",
    );
    fs::write(&change_path, impossible).unwrap();
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "abandoned change has accepted/published/disclosed metadata",
        ));
    let restored = fs::read_to_string(&change_path).unwrap().replace(
        "\"accepted_at\": \"2026-06-30T00:00:00Z\"",
        "\"accepted_at\": null",
    );
    fs::write(&change_path, restored).unwrap();

    fs::write(
        repo.join(".canopy/repo.json"),
        "{\"name\":\"demo\",\"format\":\"canopy-mvp-1\",\"active_change\":\"bad\"}\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "active change points to abandoned",
        ));
}

#[test]
fn abandoned_changes_cannot_be_reproposed_or_accepted() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("maybe.txt"), "maybe\n").unwrap();
    run(&repo, &["change", "start", "Terminal"]);
    run(&repo, &["file", "add", "maybe.txt"]);
    run(&repo, &["change", "propose", "Terminal"]);
    run(&repo, &["change", "abandon", "Terminal"]);

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "propose", "Terminal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("abandoned and cannot be proposed"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "accept", "Terminal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("abandoned and cannot be accepted"));
}

#[test]
fn abandon_retry_repairs_partial_abandoned_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("bad.txt"), "bad\n").unwrap();
    run(&repo, &["change", "start", "Partial"]);
    run(&repo, &["file", "add", "bad.txt"]);

    let change_path = repo.join(".canopy/changes/partial.json");
    let abandoned = fs::read_to_string(&change_path)
        .unwrap()
        .replace("\"status\": \"active\"", "\"status\": \"abandoned\"");
    fs::write(&change_path, abandoned).unwrap();

    run(&repo, &["change", "abandon", "Partial"]);
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .success();
    let private = temp.path().join("partial-private");
    run(
        &repo,
        &[
            "projection",
            "materialize",
            "private",
            private.to_str().unwrap(),
        ],
    );
    assert!(!private.join("bad.txt").exists());
}

#[test]
fn malformed_workspace_ops_fail_doctor_and_abandon_replay() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    run(temp.path(), &["init", repo.to_str().unwrap()]);
    fs::write(repo.join("base.txt"), "base\n").unwrap();
    fs::write(repo.join("bad.txt"), "bad\n").unwrap();
    run(&repo, &["change", "start", "Base malformed"]);
    run(&repo, &["file", "add", "base.txt"]);
    run(&repo, &["change", "finish", "Base malformed"]);
    run(&repo, &["change", "start", "Malformed"]);
    run(&repo, &["file", "add", "bad.txt"]);

    let ops_path = repo.join(".canopy/workspace-ops.json");
    let malformed = fs::read_to_string(&ops_path)
        .unwrap()
        .replace("\"content\": \"base\\n\",", "");
    fs::write(&ops_path, malformed).unwrap();

    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["doctor"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("malformed workspace operation"));
    Command::new(env!("CARGO_BIN_EXE_cnp"))
        .current_dir(&repo)
        .args(["change", "abandon", "Malformed"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("malformed workspace operation"));
}
