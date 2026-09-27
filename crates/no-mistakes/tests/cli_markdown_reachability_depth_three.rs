use std::path::PathBuf;
use std::process::Command;

#[test]
fn check_enforces_three_hop_index_routes_through_real_markdown_files() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-reachability/depth-three"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let files = findings
        .iter()
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files, ["domain/deeper/deep.md", "first.md", "second.md"]);
    assert!(findings
        .iter()
        .all(|finding| finding["rule"] == "markdown-reachability"));
}
