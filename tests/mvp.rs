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
