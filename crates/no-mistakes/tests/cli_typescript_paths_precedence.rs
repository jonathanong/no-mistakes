use std::path::PathBuf;
use std::process::Command;

#[test]
fn dependencies_cli_matches_typescript_paths_precedence() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/paths-precedence"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "dependencies",
            "src/entry.ts",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["files"],
        serde_json::json!([
            {"path":"src/exact.ts","depth":1,"via":["import"]},
            {"path":"src/first-tie/value/detail.ts","depth":1,"via":["import"]},
            {"path":"src/longest-prefix/pecific/detail.ts","depth":1,"via":["import"]},
            {"path":"src/replacement/value.ts","depth":1,"via":["import"]},
            {"module":"shadowed/value","depth":1,"via":["import"]}
        ])
    );
}
