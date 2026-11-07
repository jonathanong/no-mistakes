use serde_json::Value;
use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 37] = [
    "const-load",
    "nested-async-return",
    "owned-object-return",
    "recursive-parameter",
    "nested-recursion",
    "same-file-call-chain",
    "options-root",
    "assignment-target-options",
    "assignment-read-options",
    "glob-options-flow",
    "disposable-root",
    "rewritten-spread",
    "hook-env-restore",
    "barrel-route",
    "control-direct",
    "control-imported",
    "control-wrapper",
    "arbitrary-parameter",
    "arbitrary-options",
    "arbitrary-recursion",
    "mixed-callers",
    "mixed-live-callers",
    "arbitrary-hook-env",
    "arbitrary-rewritten-spread",
    "mutable-load",
    "unknown-load",
    "consumed-const-load",
    "recursive-parent",
    "captured-arbitrary",
    "barrel-escaped",
    "recursive-unsafe-sibling",
    "mutated-options",
    "mutated-object",
    "method-mutated-options",
    "new-mutated-options",
    "reassigned-getter",
    "absolute-glob-pattern",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "bounded-provenance", name)
}

#[test]
fn bounded_provenance_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .into_iter()
        .map(|name| {
            let root = root(name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_bounded_provenance",
        &cases,
    );
}

fn report(name: &str) -> Value {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root(name))
        .output()
        .unwrap();
    assert!(output.status.success(), "{name}: {output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn advisory_targets(report: &Value) -> Vec<&str> {
    let mut targets: Vec<_> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == RULE)
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    targets.sort();
    targets
}

#[test]
fn bounded_supported_forms_and_controls_stay_clean() {
    for name in [
        "const-load",
        "nested-async-return",
        "owned-object-return",
        "recursive-parameter",
        "nested-recursion",
        "same-file-call-chain",
        "options-root",
        "assignment-read-options",
        "disposable-root",
        "barrel-route",
        "control-direct",
        "control-imported",
        "control-wrapper",
    ] {
        let report = report(name);
        assert!(advisory_targets(&report).is_empty(), "{name}: {report}");
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{name}: {report}"
        );
    }
}

#[test]
fn bounded_forms_without_caller_proof_stay_advisory() {
    for (name, expected) in [
        ("glob-options-flow", vec!["unresolved root"]),
        ("hook-env-restore", vec!["unresolved root"]),
        ("rewritten-spread", vec!["runtime asset", "unresolved root"]),
    ] {
        let report = report(name);
        assert_eq!(advisory_targets(&report), expected, "{name}: {report}");
    }
}

#[test]
fn bounded_arbitrary_roots_and_escaped_callers_stay_advisory() {
    for name in [
        "arbitrary-parameter",
        "arbitrary-options",
        "arbitrary-recursion",
        "mixed-callers",
        "mixed-live-callers",
        "arbitrary-hook-env",
        "arbitrary-rewritten-spread",
        "mutable-load",
        "unknown-load",
        "consumed-const-load",
        "recursive-parent",
        "captured-arbitrary",
        "barrel-escaped",
        "recursive-unsafe-sibling",
        "mutated-options",
        "assignment-target-options",
        "mutated-object",
        "method-mutated-options",
        "new-mutated-options",
        "reassigned-getter",
        "absolute-glob-pattern",
    ] {
        let report = report(name);
        assert!(
            advisory_targets(&report).contains(&"unresolved root"),
            "{name}: {report}"
        );
    }
}

#[test]
fn bounded_assignment_targets_each_invalidate_options_root() {
    let report = report("assignment-target-options");
    assert_eq!(advisory_targets(&report), vec!["unresolved root"; 10]);
}
