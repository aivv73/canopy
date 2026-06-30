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
        .stdout(predicate::str::contains("Handle: change/inspect-me"));
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
