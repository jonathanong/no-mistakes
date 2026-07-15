#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
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
fn banned_paths_reports_tracked_artifacts_below_source_skip_directories() {
    let fixture = gitignore_fixture::materialize("banned-paths-source-skips");
    let root = fixture.path();
    assert!(git(root, &["init", "-q", "--initial-branch=main"]));
    assert!(git(root, &["add", "."]));

    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let files = value["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "banned-paths")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(out.status.code(), Some(1), "{body}");
    assert_eq!(
        files,
        vec![
            "build/blocked.patch",
            "dist/blocked.patch",
            "fixtures/blocked.patch",
            "nested/blocked.patch",
            "target/blocked.patch",
        ]
    );
}
