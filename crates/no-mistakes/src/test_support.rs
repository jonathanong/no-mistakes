mod gitignore_fixture;

pub(crate) use gitignore_fixture::{materialize_gitignore_fixture, materialize_saved_fixture};

use std::path::Path;
use std::process::Command;

pub(crate) fn git_init(root: &Path) {
    run_git(root, &["init", "-q", "--initial-branch=main"]);
}

pub(crate) fn git_add_all(root: &Path) {
    run_git(root, &["add", "."]);
}

pub(crate) fn git_add_force(root: &Path, paths: &[&str]) {
    let mut args = vec!["add", "-f", "--"];
    args.extend_from_slice(paths);
    run_git(root, &args);
}

pub(crate) fn git_config(root: &Path, key: &str, value: &Path) {
    run_git(root, &["config", key, value.to_str().unwrap()]);
}

fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}
