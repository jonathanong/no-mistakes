use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::tempdir;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes_core::codebase::ts_resolver::normalize_path(
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
fn tests_plan_json_outputs_impacted_tests() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], false);

    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 2);
    let mut names: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);

    let a_test = selected
        .iter()
        .find(|t| t["test_file"] == "a.test.mts")
        .unwrap();
    assert_eq!(a_test["confidence"], "high");

    let dynamic_test = selected
        .iter()
        .find(|t| t["test_file"] == "dynamic.test.mts")
        .unwrap();
    assert_eq!(dynamic_test["confidence"], "medium");
}

#[test]
fn tests_plan_paths_outputs_newline_separated_paths() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--format",
        "paths",
    ]);

    assert!(output.status.success());
    let paths_str = stdout(&output);
    let mut paths: Vec<&str> = paths_str.lines().collect();
    paths.sort();
    assert_eq!(paths, vec!["a.test.mts", "dynamic.test.mts"]);
}

#[test]
fn tests_plan_md_outputs_markdown_table() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--format",
        "md",
    ]);

    assert!(output.status.success());
    let md = stdout(&output);
    assert!(md.contains("# 🧪 Test Impact Analysis"));
    assert!(md.contains("## Selected Tests (Total: 2)"));
    assert!(md.contains("| Test File | Confidence | Reason / Impact Chain |"));
    assert!(md.contains("a.test.mts"));
    assert!(md.contains("dynamic.test.mts"));
}

#[test]
fn tests_plan_fallback_on_package_json() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("Global configuration file changed"));

    let selected = plan["selected_tests"].as_array().unwrap();
    // It should select all tests in this fixture
    assert_eq!(selected.len(), 2);
    let mut names: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);
    for t in selected {
        assert_eq!(t["confidence"], "high");
    }
}

#[test]
fn tests_why_displays_dependency_path() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "why",
        "a.test.mts",
        "--changed",
        "c.mts",
        "--root",
        root.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Path from `c.mts` to `a.test.mts`"));
    assert!(text.contains("c.mts"));
    assert!(text.contains("b.mts"));
    assert!(text.contains("a.mts"));
    assert!(text.contains("a.test.mts"));
}

#[test]
fn tests_comment_formats_markdown() {
    let tmp = tempdir().unwrap();
    let plan_file = tmp.path().join("plan.json");

    let sample_plan = serde_json::json!({
        "selected_tests": [
            {
                "test_file": "a.test.mts",
                "confidence": "high",
                "reasons": [
                    {
                        "changed_file": "c.mts",
                        "path": ["c.mts", "b.mts", "a.mts", "a.test.mts"],
                        "via": ["Import", "Import", "Import"]
                    }
                ]
            }
        ],
        "warnings": [],
        "fallback_triggered": false,
        "fallback_reason": null
    });

    fs::write(&plan_file, serde_json::to_string(&sample_plan).unwrap()).unwrap();

    let output = run(&["tests", "comment", plan_file.to_str().unwrap()]);

    assert!(output.status.success());
    let md = stdout(&output);
    assert!(md.contains("# 🧪 Test Impact Analysis"));
    assert!(md.contains("a.test.mts"));
    assert!(md.contains("🟢 High"));
}

#[test]
fn tests_graph_mermaid_outputs_flowchart() {
    let tmp = tempdir().unwrap();
    let plan_file = tmp.path().join("plan.json");

    let sample_plan = serde_json::json!({
        "selected_tests": [
            {
                "test_file": "a.test.mts",
                "confidence": "high",
                "reasons": [
                    {
                        "changed_file": "c.mts",
                        "path": ["c.mts", "b.mts", "a.mts", "a.test.mts"],
                        "via": ["Import", "Import", "Import"]
                    }
                ]
            }
        ],
        "warnings": [],
        "fallback_triggered": false,
        "fallback_reason": null
    });

    fs::write(&plan_file, serde_json::to_string(&sample_plan).unwrap()).unwrap();

    // 1. Mermaid
    let output_mermaid = run(&[
        "tests",
        "graph",
        plan_file.to_str().unwrap(),
        "--format",
        "mermaid",
    ]);

    assert!(output_mermaid.status.success());
    let mermaid = stdout(&output_mermaid);
    assert!(mermaid.contains("graph TD"));
    assert!(mermaid.contains("classDef changed"));
    assert!(mermaid.contains("classDef test"));
    assert!(mermaid.contains("c.mts"));
    assert!(mermaid.contains("a.test.mts"));

    // 2. JSON
    let output_json = run(&[
        "tests",
        "graph",
        plan_file.to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output_json.status.success());
    let json_str = stdout(&output_json);
    let graph: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(graph["nodes"].as_array().unwrap().len() >= 4);
    let edges = graph["edges"].as_array().unwrap();
    assert!(edges.iter().any(|e| e["via"] == "Import"));
}

#[test]
fn tests_plan_nested_package_json_does_not_trigger_fallback() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "nested/package.json",
        "--json",
    ]);

    assert!(output.status.success());
    let json_str = stdout(&output);
    let plan: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
}

#[test]
fn tests_plan_head_requires_base() {
    let root = fixture("tests-impact");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--head",
        "main",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stderr.contains("the following required arguments were not provided:"));
    assert!(stderr.contains("--base"));
}
