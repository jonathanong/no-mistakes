use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> tempfile::TempDir {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    crate::saved_fixture::materialize_path(&root)
}

fn plan(root: &Path, jobs: &str, extra: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "vitest", "--root"])
        .arg(root)
        .args(["--format", "json", "--jobs", jobs])
        .args(extra)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn selected_files(report: &Value) -> Vec<&str> {
    report["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect()
}

fn summary(report: &Value) -> Value {
    json!({
        "selected": selected_files(report),
        "fallback": report["fallback_triggered"],
        "reason": report["fallback_reason"],
        "warnings": report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|warning| warning["type"].as_str())
            .collect::<Vec<_>>(),
    })
}

const ALL_TESTS: [&str; 3] = [
    "tests/browser.test.ts",
    "tests/example.test.ts",
    "tests/reader.test.ts",
];

#[test]
fn changes_matches_literal_ids_and_supersedes_only_the_configured_path_trigger() {
    let fixture = fixture();
    let cases: &[(&str, &[&str])] = &[
        ("changes_only", &["--changes", "reports/matched.json"]),
        (
            "both_inputs",
            &[
                "--changes",
                "reports/matched.json",
                "--changed-file",
                "data/example.json",
                "--changed-file",
                "src/example.ts",
            ],
        ),
        ("path_only", &["--changed-file", "data/example.json"]),
        ("without_input", &[]),
        ("targeted", &["--changes", "reports/targeted.json"]),
        ("missing_trigger", &["--changes", "reports/unmatched.json"]),
        ("empty_trigger", &["--changes", "reports/empty-target.json"]),
        (
            "empty_changes",
            &[
                "--changes",
                "reports/empty.json",
                "--changed-file",
                "data/example.json",
            ],
        ),
        (
            "path_and_changes",
            &[
                "--changed-file",
                "data/example.json",
                "--changes",
                "reports/matched.json",
            ],
        ),
    ];
    let mut summaries = serde_json::Map::new();
    for &(name, args) in cases {
        let report = plan(fixture.path(), "1", args);
        assert_eq!(report, plan(fixture.path(), "2", args), "{name}");
        summaries.insert(name.into(), summary(&report));
    }
    let matching = plan(fixture.path(), "1", cases[0].1);
    let reasons = matching["selected_tests"][0]["reasons"].as_array().unwrap();
    assert_eq!(
        reasons
            .iter()
            .map(|reason| reason["via"][0].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "changes:fixtures:example.a+b",
            "changes:fixtures:new.id",
            "changes:fixtures:old.id",
            "changes:text:site.title",
        ]
    );
    let targeted = plan(fixture.path(), "1", cases[4].1);
    assert_eq!(
        targeted["selected_tests"][0]["targets"][0]["project"],
        "browser"
    );
    let empty_target = plan(fixture.path(), "1", cases[6].1);
    assert_eq!(selected_files(&empty_target), ALL_TESTS);
    assert!(empty_target["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("has no discovered tests for vitest project `empty`"));
    assert_eq!(summaries["path_only"]["selected"], json!(ALL_TESTS));
    assert_eq!(
        summaries["path_and_changes"]["selected"],
        json!(["tests/reader.test.ts"])
    );
    insta::assert_snapshot!(
        "changes_routing",
        serde_json::to_string_pretty(&summaries).unwrap()
    );
}

#[test]
fn invalid_changes_input_runs_the_full_discovered_suite_despite_plan_limits() {
    let fixture = fixture();
    let mut summaries = serde_json::Map::new();
    for input in [
        "bad-category.json",
        "bad-id.json",
        "incomplete.json",
        "invalid.txt",
        "missing.json",
        "no-version.json",
        "no-complete.json",
        "no-unclassified.json",
        "unclassified.json",
        "unknown.json",
        "unsupported.json",
    ] {
        let path = format!("reports/{input}");
        let args = ["--changes", &path, "--limit-files", "1"];
        let report = plan(fixture.path(), "1", &args);
        assert_eq!(report, plan(fixture.path(), "2", &args), "{input}");
        assert_eq!(selected_files(&report), ALL_TESTS, "{input}");
        assert_eq!(report["fallback_triggered"], true, "{input}");
        assert_eq!(report["warnings"][0]["type"], "changes-fallback", "{input}");
        summaries.insert(input.into(), summary(&report));
    }
    insta::assert_snapshot!(
        "changes_safety",
        serde_json::to_string_pretty(&summaries).unwrap()
    );
}

#[test]
fn changes_rejects_direct_test_owner_mode() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["tests", "plan", "vitest", "--root"])
        .arg(fixture.path())
        .args(["--direct-test-owner", "--changes", "reports/matched.json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("--direct-test-owner") && error.contains("--changes"),
        "{error}"
    );
}
