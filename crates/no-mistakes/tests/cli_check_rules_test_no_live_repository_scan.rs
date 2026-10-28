use serde_json::Value;
use std::path::PathBuf;
use std::process::Output;

const RULE: &str = "test-no-live-repository-scan";

fn root() -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "fixture", "")
}

fn check(config: &str) -> (Output, Value) {
    let root = root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, report)
}

fn located(items: &Value) -> Vec<String> {
    let mut found: Vec<String> = items
        .as_array()
        .map(|items| items.as_slice())
        .unwrap_or_default()
        .iter()
        .filter(|item| item["rule"] == RULE)
        .map(|item| format!("{}:{}", item["file"].as_str().unwrap(), item["line"]))
        .collect();
    found.sort();
    found
}

#[test]
fn default_policy_json_snapshot() {
    let root = root();
    let cases: Vec<(&str, PathBuf, PathBuf)> = [
        ("default", ".no-mistakes.yml"),
        ("unresolved_report", "report.yml"),
        ("custom_fixture_dirs", "fixture-dirs.yml"),
        ("glob_root", "glob-root.yml"),
        ("extra_enumerator", "enumerators.yml"),
        ("severity_advisory", "advisory.yml"),
    ]
    .into_iter()
    .map(|(label, config)| (label, root.clone(), root.join(config)))
    .collect();
    crate::check_json_snapshot::assert_saved_fixture("test_no_live_repository_scan", &cases);
}

#[test]
fn rejects_scans_reachable_from_vitest_roots_and_allows_bounded_input() {
    let (output, report) = check(".no-mistakes.yml");
    assert_eq!(output.status.code(), Some(1), "{report}");
    let findings = located(&report["rules"]);
    for expected in [
        "tests/direct-module-scope.test.ts:7",
        "tests/aliased-call.test.ts:7",
        "tests/aliased-call.test.ts:8",
        "tests/aliased-call.test.ts:9",
        "tests/aliased-call.test.ts:10",
        "tests/aliased-call.test.ts:11",
        "tests/glob-package.test.ts:5",
        "tests/glob-package.test.ts:6",
        "helpers/workflows.ts:7",
        "helpers/policy-scan.ts:5",
        "helpers/scan-on-import.ts:3",
        "tests/path-forms.test.ts:13",
        "tests/path-forms.test.ts:14",
        "tests/path-forms.test.ts:15",
        "tests/path-forms.test.ts:16",
        "tests/path-forms.test.ts:17",
        "tests/path-forms.test.ts:21",
    ] {
        assert!(
            findings.contains(&expected.to_string()),
            "{expected}: {findings:?}"
        );
    }
    for allowed in [
        "bounded-fixtures",
        "owned-temp-root",
        "suppressed",
        "scripts/live-scan",
    ] {
        assert!(
            findings.iter().all(|f| !f.contains(allowed)),
            "{allowed}: {findings:?}"
        );
    }
    // Unresolved provenance is advisory by default, so it never passes silently.
    assert_eq!(
        located(&report["advisories"]),
        [
            "helpers/list-files.ts:4",
            "tests/path-forms.test.ts:23",
            "tests/path-forms.test.ts:24",
            "tests/path-forms.test.ts:25",
            "tests/path-forms.test.ts:26",
            "tests/path-forms.test.ts:27",
            "tests/unresolved-root.test.ts:8",
        ]
    );
}

#[test]
fn unresolved_root_can_fail_the_run_and_severity_demotes_everything() {
    let (_, report) = check("report.yml");
    let findings = located(&report["rules"]);
    assert!(
        findings.contains(&"helpers/list-files.ts:4".to_string()),
        "{findings:?}"
    );
    let (output, report) = check("advisory.yml");
    assert!(output.status.success(), "{report}");
    assert!(located(&report["rules"]).is_empty(), "{report}");
    assert!(located(&report["advisories"]).len() > 10, "{report}");
}

#[test]
fn documented_options_narrow_and_extend_the_policy() {
    let (_, report) = check("fixture-dirs.yml");
    let findings = located(&report["rules"]);
    // `fixtureDirs` replaces the defaults, so the old fixtures directory is live.
    assert!(
        findings.contains(&"tests/bounded-fixtures.test.ts:7".to_string()),
        "{findings:?}"
    );
    let (_, report) = check("glob-root.yml");
    let findings = located(&report["rules"]);
    assert_eq!(findings, ["helpers/policy-scan.ts:5"], "{findings:?}");
    let (_, report) = check("enumerators.yml");
    let findings = located(&report["rules"]);
    assert!(
        findings.contains(&"tests/extra-enumerator.test.ts:4".to_string()),
        "{findings:?}"
    );
    let (_, report) = check(".no-mistakes.yml");
    assert!(!located(&report["rules"]).contains(&"tests/extra-enumerator.test.ts:4".to_string()));
}

#[test]
fn invalid_options_are_configuration_errors() {
    let (output, _) = check("invalid.yml");
    let text = String::from_utf8_lossy(&output.stderr).to_string()
        + &String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success());
    assert!(
        text.contains("fixtureDirs must list path segments"),
        "{text}"
    );
}
