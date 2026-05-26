use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis")
            .join(name),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn test_impact_file_only_finds_tests() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "utils.mts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    let test_files: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert!(
        test_files.contains(&"service.test.mts"),
        "should find service.test.mts: {test_files:?}"
    );
    assert!(
        test_files.contains(&"other.test.mts"),
        "should find other.test.mts: {test_files:?}"
    );
}

#[test]
fn test_impact_multiple_entrypoints_union() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "service.mts",
        "other.mts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    let test_files: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert!(
        test_files.contains(&"service.test.mts"),
        "should find service.test.mts: {test_files:?}"
    );
    assert!(
        test_files.contains(&"other.test.mts"),
        "should find other.test.mts: {test_files:?}"
    );
}

#[test]
fn test_impact_json_output_matches_schema() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "service.mts",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(plan["selected_tests"].is_array());
    assert!(plan["warnings"].is_array());
    assert_eq!(plan["fallback_triggered"], false);

    let first = &plan["selected_tests"][0];
    assert!(first["test_file"].is_string());
    assert!(first["confidence"].is_string());
    assert!(first["reasons"].is_array());
    let reason = &first["reasons"][0];
    assert!(reason["changed_file"].is_string());
    assert!(reason["path"].is_array());
    assert!(reason["via"].is_array());
}

#[test]
fn test_impact_paths_format() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "service.mts",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    let paths: Vec<&str> = out.trim().lines().collect();
    assert!(
        paths.contains(&"service.test.mts"),
        "paths format should list service.test.mts: {paths:?}"
    );
}
