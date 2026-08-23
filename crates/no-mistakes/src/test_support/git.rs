use std::path::Path;
use std::process::Command;

pub(crate) fn git_init(root: &Path) {
    run_git(root, &["init", "-q", "--initial-branch=main"]);
}

pub(crate) fn git_add_all(root: &Path) {
    run_git(root, &["add", "."]);
}

pub(super) fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = format!("git {} failed: {stderr}", args.join(" "));
    assert!(output.status.success(), "{detail}");
}
