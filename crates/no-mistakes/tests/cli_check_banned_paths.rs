#[allow(dead_code)]
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn git(root: &std::path::Path, args: &[&str]) -> bool {
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

#[test]
fn banned_paths_cli_reports_only_tracked_files() {
    let dir = gitignore_fixture::materialize_saved("banned-paths-tracked-only");
    let root = dir.path();
    assert!(git(root, &["init", "-q", "--initial-branch=main"]));
    assert!(git(root, &["add", ".no-mistakes.yml", "tracked.patch"]));
    std::fs::rename(
        root.join("gitignore-after.fixture"),
        root.join(".gitignore"),
    )
    .unwrap();
    assert!(git(root, &["add", ".gitignore"]));

    let output = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let banned_files = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "banned-paths")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(!output.status.success());
    assert_eq!(banned_files, ["tracked.patch"]);
}
