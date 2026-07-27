#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn git(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .status()
        .unwrap()
        .success()
}

fn check(root: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn commit_fixture(root: &Path) {
    assert!(git(root, &["init", "-q"]));
    assert!(git(root, &["add", "."]));
    assert!(git(
        root,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "init",
        ]
    ));
}

#[test]
fn agents_md_max_size_skips_gitignored_files() {
    let fixture = gitignore_fixture::materialize("cli-check-rules-agents-md-max-size");
    let root = fixture.path();
    commit_fixture(root);

    let output = check(root);
    assert!(
        output.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&output)
    );
}

#[test]
fn rust_no_inline_tests_skips_gitignored_files() {
    let fixture = gitignore_fixture::materialize("cli-check-rules-rust-no-inline-tests");
    let root = fixture.path();
    commit_fixture(root);

    let output = check(root);
    assert!(
        output.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&output)
    );
}
