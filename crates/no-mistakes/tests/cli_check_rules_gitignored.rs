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

fn check(root: &Path, config: &str) -> Output {
    let config_file = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config_file.path(), config).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config_file.path())
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
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("ignored")).unwrap();
    std::fs::write(root.join("ignored/AGENTS.md"), "line\n".repeat(300)).unwrap();
    std::fs::write(root.join("CLAUDE.md"), "# ok\n").unwrap();
    std::fs::write(root.join(".gitignore"), "ignored/\n").unwrap();
    commit_fixture(root);

    let output = check(
        root,
        "rules:\n  - rule: agents-md-max-size\n    scope: repository\n    options:\n      maxLines: 5\n",
    );
    assert!(
        output.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&output)
    );
}

#[test]
fn rust_no_inline_tests_skips_gitignored_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("generated")).unwrap();
    std::fs::write(
        root.join("generated/lib.rs"),
        "#[cfg(test)]\nmod tests {\n}\n",
    )
    .unwrap();
    std::fs::write(root.join("clean.rs"), "pub fn ok() {}\n").unwrap();
    std::fs::write(root.join(".gitignore"), "generated/\n").unwrap();
    commit_fixture(root);

    let output = check(
        root,
        "rules:\n  - rule: rust-no-inline-tests\n    scope: repository\n",
    );
    assert!(
        output.status.success(),
        "gitignored files must not be flagged: {}",
        stdout(&output)
    );
}
