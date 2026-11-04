use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config-fragments-parity")
        .join(name)
}

fn report(root: &Path, command: &[&str]) -> (Option<i32>, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(command)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap();
    if command.starts_with(&["config", "resolve"]) || command.starts_with(&["tests", "plan"]) {
        return (
            output.status.code(),
            serde_json::from_slice(&output.stdout).unwrap(),
        );
    }
    let normalized = crate::check_json_snapshot::normalize_report(
        &String::from_utf8(output.stdout).unwrap(),
        &[root.to_string_lossy().into_owned()],
    );
    (
        output.status.code(),
        serde_json::from_str(&normalized).unwrap(),
    )
}

#[test]
fn split_config_check_and_dependency_output_matches_single_file() {
    let single = fixture("single");
    let split = fixture("split");
    for command in [
        vec!["check", "--format", "json"],
        vec!["dependents", "src/value.ts", "--format", "json"],
    ] {
        assert_eq!(
            report(&single, &command),
            report(&split, &command),
            "{command:?}"
        );
    }
}

#[test]
fn check_json_snapshot_config_fragments() {
    crate::check_json_snapshot::assert_saved_fixture(
        "config_fragments",
        &[
            ("single", fixture("single"), PathBuf::new()),
            ("split", fixture("split"), PathBuf::new()),
            (
                "provenance",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../test-cases/config-v2/split-provenance/fixture/anchors"),
                PathBuf::new(),
            ),
        ],
    );
}

#[test]
fn fragment_diagnostics_honor_physical_file_suppressions() {
    for mode in ["file", "line", "next-line"] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/config-fragments-suppression")
            .join(mode);
        let (status, value) = report(&root, &["check", "--format", "json"]);
        assert_eq!(status, Some(0), "{mode}: {value}");
    }
}

#[test]
fn declared_view_retains_inline_values_and_deterministic_entry_order() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/config-fragments/names");
    let (status, config) = report(&root, &["config", "resolve", "--view", "declared"]);
    assert_eq!(status, Some(0), "{config}");
    let names: Vec<_> = config["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["root", "alpha", "beta"]);
    assert!(config.get("filesystem").is_none());
    assert_eq!(config["effort"], "low");
    let empty = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/config/declared-empty");
    assert_eq!(
        report(&empty, &["config", "resolve", "--view", "declared"]),
        (Some(0), serde_json::json!({}))
    );
}

#[test]
fn split_and_single_file_trigger_plans_are_identical() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan");
    let command = [
        "tests",
        "plan",
        "vitest",
        "--changed-file",
        "migrations/001.sql",
        "--format",
        "json",
    ];
    let single = report(&base.join("target-scoped-triggers"), &command);
    let split = report(&base.join("config-fragments"), &command);
    assert_eq!(single, split);
    assert_eq!(split.0, Some(0));
    assert_eq!(
        split.1["selected_tests"][0]["test_file"],
        "src/db/db.test.ts"
    );
    assert_eq!(
        split.1["selected_tests"][0]["targets"][0]["project"],
        "database"
    );
}

#[test]
fn deprecated_fragment_warnings_name_the_declaring_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config-fragments-aliases/all-frameworks");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["config", "resolve", "--root"])
        .arg(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let warnings = String::from_utf8(output.stderr).unwrap().replace('\\', "/");
    assert!(
        warnings.contains(".no-mistakes/testPlan/all.yml"),
        "{warnings}"
    );
    assert!(
        warnings.contains("playwright.dependencies") && warnings.contains("vitest.dependencies")
    );
}
