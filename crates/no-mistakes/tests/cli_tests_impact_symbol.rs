use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
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
fn test_impact_symbol_opt_in_limits_to_symbol_consumers() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "utils.mts#parseDate",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let test_files: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(test_files, vec!["other.test.mts"]);
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["changed_file"],
        "utils.mts#parseDate"
    );
}

#[test]
fn dependents_symbols_outputs_symbol_nodes_when_opted_in() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "dependents",
        "utils.mts#parseDate",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let files = value["files"].as_array().unwrap();
    assert!(files
        .iter()
        .any(|file| file["file"] == "other.mts" && file["symbol"] == "parse"));
    assert!(files.iter().any(|file| file["path"] == "other.test.mts"));
    assert!(!files.iter().any(|file| file["path"] == "service.test.mts"));
}

#[test]
fn dependencies_symbols_paths_render_file_hash_symbol() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "dependencies",
        "other.mts#parse",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "utils.mts#parseDate\n");
}

#[test]
fn dependencies_symbols_handles_default_and_variable_exports() {
    let root = fixture("symbol-export");
    let default_output = run(&[
        "dependencies",
        "default-consumer.mts#run",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);
    assert!(
        default_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&default_output.stderr)
    );
    assert_eq!(stdout(&default_output), "default-source.mts#default\n");

    let variable_output = run(&[
        "dependencies",
        "uses-alpha-variable.mts#value",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--format",
        "paths",
    ]);
    assert!(
        variable_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&variable_output.stderr)
    );
    assert_eq!(stdout(&variable_output), "source.mts#alpha\n");
}

#[test]
fn dependencies_symbols_handles_local_alias_barrel_and_helper_calls() {
    let root = fixture("symbol-export");
    for (entrypoint, expected) in [
        ("aliased-local.mts#publicLocal", "source.mts#alpha\n"),
        ("local-barrel.mts#alpha", "source.mts#alpha\n"),
        ("helper-chain.mts#run", "source.mts#alpha\n"),
        ("namespace-consumer.mts#value", "source.mts#alpha\n"),
        ("star-barrel.mts#alpha", "source.mts#alpha\n"),
    ] {
        let output = run(&[
            "dependencies",
            entrypoint,
            "--root",
            root.to_str().unwrap(),
            "--symbols",
            "--format",
            "paths",
        ]);
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(stdout(&output), expected, "{entrypoint}");
    }
}

#[test]
fn test_impact_symbol_test_entrypoint_keeps_runnable_test_path_plain() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "impact",
        "other.test.mts#helper",
        "--root",
        root.to_str().unwrap(),
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = &plan["selected_tests"][0];
    assert_eq!(selected["test_file"], "other.test.mts");
    assert_eq!(
        selected["reasons"][0]["changed_file"],
        "other.test.mts#helper"
    );
}

#[test]
fn tests_plan_symbol_test_entrypoint_keeps_runnable_test_path_plain() {
    let root = fixture("tests-impact-symbol");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--entrypoint",
        "other.test.mts#helper",
        "--symbols",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = &plan["selected_tests"][0];
    assert_eq!(selected["test_file"], "other.test.mts");
    assert_eq!(
        selected["reasons"][0]["changed_file"],
        "other.test.mts#helper"
    );
    assert_eq!(selected["reasons"][0]["path"][0], "other.test.mts#helper");
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
